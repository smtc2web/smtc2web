; NSIS installer hooks for smtc2web
; - Registers the install directory in PATH on install
; - Removes it from PATH on uninstall
; - Optionally deletes user data on uninstall
;
; PATH safety notes:
; NSIS ReadRegStr returns an empty string and sets the error flag when the
; existing PATH value is longer than the NSIS string limit. The old code
; interpreted that as "PATH is empty" and overwrote the whole PATH with just
; the install directory, silently destroying every other entry. To prevent
; this, PATH is only modified after its real length is queried through the
; Win32 API (RegQueryValueEx). If the value is too long to handle safely
; inside NSIS, the PATH update is skipped and PATH is left untouched.
;
; Registry hive notes:
; tauri-bundler !includes this file *before* it !defines INSTALLMODE, so a
; file-scope `!if "${INSTALLMODE}" == "currentUser"` is evaluated while the
; define is still unknown. It therefore always fell through to the `!else`
; branch and wrote the per-user PATH to HKLM. The hive is now selected inside
; the hook macros instead: those are expanded in the install/uninstall
; sections, long after INSTALLMODE has been defined.
;
; Hang notes:
; A plain `SendMessage ${HWND_BROADCAST}` waits for every top-level window to
; process the message. A single window that is not pumping (hung app, blocked
; shell, etc.) freezes setup forever, which is why the WM_SETTINGCHANGE
; broadcast below carries an explicit /TIMEOUT.

!include "WinMessages.nsh"

; ---- 多语言字符串 ----
; English (always present as default)
LangString smtc2_registering_path  ${LANG_ENGLISH}   "Registering smtc2web in PATH..."
LangString smtc2_removing_path     ${LANG_ENGLISH}   "Removing smtc2web from PATH..."
LangString smtc2_deleting_data     ${LANG_ENGLISH}   "Deleting smtc2web user data..."

; Simplified Chinese
!ifdef LANG_SIMPCHINESE
LangString smtc2_registering_path  ${LANG_SIMPCHINESE} "正在将 smtc2web 注册到 PATH..."
LangString smtc2_removing_path     ${LANG_SIMPCHINESE} "正在从 PATH 移除 smtc2web..."
LangString smtc2_deleting_data     ${LANG_SIMPCHINESE} "正在删除 smtc2web 用户数据..."
!endif

; Traditional Chinese
!ifdef LANG_TRADCHINESE
LangString smtc2_registering_path  ${LANG_TRADCHINESE} "正在將 smtc2web 註冊到 PATH..."
LangString smtc2_removing_path     ${LANG_TRADCHINESE} "正在從 PATH 移除 smtc2web..."
LangString smtc2_deleting_data     ${LANG_TRADCHINESE} "正在刪除 smtc2web 使用者資料..."
!endif

; Declare StrFunc helpers (StrFunc.nsh is already included by the main installer)
${StrStr}
${UnStrStr}
${StrRep}
${UnStrRep}

; Emits one PATH add/remove pair for a single registry hive, so the callers
; never have to depend on a compile-time INSTALLMODE check to pick a hive.
!macro SMTC_PATH_HELPERS ROOT ROOTNUM SUBKEY
  ; Appends the directory on the stack ($0) to the PATH value.
  ; Uses RegQueryValueEx to learn the real value length first, so an
  ; over-long PATH is skipped instead of being overwritten.
  Function SMTC_AddToPath${ROOT}
    Exch $0
    Push $1
    Push $2
    Push $3
    Push $4
    Push $5

    StrCpy $1 ""
    System::Call "advapi32::RegOpenKey(i ${ROOTNUM}, t'${SUBKEY}', *i.r2) i.r3"
    IntCmp $3 0 0 smtc_add_done_${ROOT} smtc_add_done_${ROOT}
    System::Call "advapi32::RegQueryValueEx(i $2, t'Path', i 0, i 0, i 0, *i 0 r4) i.r3"
    System::Call "advapi32::RegCloseKey(i $2)"
    ; $3 = 0 (value exists), 2 (value not found), 234 (exists, size in $4)
    IntCmp $3 2 smtc_add_missing_${ROOT}
    IntCmp $3 234 smtc_add_checklen_${ROOT}
    IntCmp $3 0 smtc_add_read_${ROOT}
    Goto smtc_add_done_${ROOT}
  smtc_add_checklen_${ROOT}:
    ; Required size (bytes) is in $4. An NSIS string holds up to
    ; NSIS_MAX_STRLEN-1 characters (1024 characters / 2048 bytes here).
    IntOp $5 ${NSIS_MAX_STRLEN} * 2
    IntCmp $4 $5 smtc_add_read_${ROOT} smtc_add_read_${ROOT} smtc_add_toolong_${ROOT}
  smtc_add_read_${ROOT}:
    ClearErrors
    ReadRegStr $1 ${ROOT} "${SUBKEY}" "Path"
    IfErrors smtc_add_done_${ROOT}
    Goto smtc_add_process_${ROOT}
  smtc_add_missing_${ROOT}:
    StrCpy $1 ""
    Goto smtc_add_process_${ROOT}
  smtc_add_process_${ROOT}:
    ; Skip when the directory is already present as a standalone entry.
    StrCpy $2 ";$1;"
    StrCpy $3 ";$0;"
    ${StrStr} $4 $2 $3
    StrCmp $4 "" 0 smtc_add_done_${ROOT}

    ; The final value must still fit in an NSIS string.
    StrLen $2 $1
    StrLen $3 $0
    IntOp $2 $2 + $3
    IntOp $2 $2 + 1
    IntCmp $2 ${NSIS_MAX_STRLEN} smtc_add_toolong_${ROOT} 0 smtc_add_toolong_${ROOT}

    StrCmp $1 "" smtc_add_empty_${ROOT}
    StrCpy $2 $1 1 -1
    StrCmp $2 ";" smtc_add_strip_${ROOT} smtc_add_nostrip_${ROOT}
  smtc_add_strip_${ROOT}:
    StrCpy $1 $1 -1
  smtc_add_nostrip_${ROOT}:
    StrCpy $0 "$1;$0"
    Goto smtc_add_write_${ROOT}
  smtc_add_empty_${ROOT}:
    StrCpy $0 $0
  smtc_add_write_${ROOT}:
    WriteRegExpandStr ${ROOT} "${SUBKEY}" "Path" $0
    Goto smtc_add_done_${ROOT}
  smtc_add_toolong_${ROOT}:
    DetailPrint "smtc2web: PATH is too long to update safely; leaving PATH unchanged"
  smtc_add_done_${ROOT}:
    Pop $5
    Pop $4
    Pop $3
    Pop $2
    Pop $1
    Pop $0
  FunctionEnd

  ; Removes the directory on the stack ($0) from the PATH value.
  Function un.SMTC_RemoveFromPath${ROOT}
    Exch $0
    Push $1
    Push $2
    Push $3
    Push $4
    Push $5
    Push $6

    StrCpy $1 ""
    System::Call "advapi32::RegOpenKey(i ${ROOTNUM}, t'${SUBKEY}', *i.r2) i.r3"
    IntCmp $3 0 0 smtc_rem_done_${ROOT} smtc_rem_done_${ROOT}
    System::Call "advapi32::RegQueryValueEx(i $2, t'Path', i 0, i 0, i 0, *i 0 r4) i.r3"
    System::Call "advapi32::RegCloseKey(i $2)"
    IntCmp $3 2 smtc_rem_done_${ROOT}
    IntCmp $3 234 smtc_rem_checklen_${ROOT}
    IntCmp $3 0 smtc_rem_read_${ROOT}
    Goto smtc_rem_done_${ROOT}
  smtc_rem_checklen_${ROOT}:
    IntOp $5 ${NSIS_MAX_STRLEN} * 2
    IntCmp $4 $5 smtc_rem_read_${ROOT} smtc_rem_read_${ROOT} smtc_rem_toolong_${ROOT}
  smtc_rem_read_${ROOT}:
    ClearErrors
    ReadRegStr $1 ${ROOT} "${SUBKEY}" "Path"
    IfErrors smtc_rem_done_${ROOT}
    Goto smtc_rem_process_${ROOT}
  smtc_rem_process_${ROOT}:
    ; Only proceed when the directory is a standalone PATH entry.
    StrCpy $2 ";$1;"
    StrCpy $3 ";$0;"
    ${UnStrStr} $4 $2 $3
    StrCmp $4 "" smtc_rem_done_${ROOT}

    ; Remove ";dir;" from the padded value, then undo the padding.
    ${UnStrRep} $5 $2 "$3" ";"
    StrCpy $5 $5 "" 1
    StrLen $6 $5
    IntOp $6 $6 - 1
    StrCpy $5 $5 $6
    ; Collapse any leftover double separators.
    ${UnStrRep} $6 $5 ";;" ";"
    ; Trim leading and trailing separators.
    StrCpy $5 $6 1 0
    StrCmp $5 ";" smtc_rem_trim_lead_${ROOT} smtc_rem_no_lead_${ROOT}
  smtc_rem_trim_lead_${ROOT}:
    StrCpy $6 $6 "" 1
  smtc_rem_no_lead_${ROOT}:
    StrCpy $5 $6 1 -1
    StrCmp $5 ";" smtc_rem_trim_tail_${ROOT} smtc_rem_no_tail_${ROOT}
  smtc_rem_trim_tail_${ROOT}:
    StrCpy $6 $6 -1
  smtc_rem_no_tail_${ROOT}:
    StrCmp $6 $1 smtc_rem_done_${ROOT}
    WriteRegExpandStr ${ROOT} "${SUBKEY}" "Path" $6
    Goto smtc_rem_done_${ROOT}
  smtc_rem_toolong_${ROOT}:
    DetailPrint "smtc2web: PATH is too long to update safely; leaving PATH unchanged"
  smtc_rem_done_${ROOT}:
    Pop $6
    Pop $5
    Pop $4
    Pop $3
    Pop $2
    Pop $1
    Pop $0
  FunctionEnd
!macroend

!insertmacro SMTC_PATH_HELPERS HKCU 0x80000001 "Environment"
!insertmacro SMTC_PATH_HELPERS HKLM 0x80000002 "SYSTEM\CurrentControlSet\Control\Session Manager\Environment"

!macro NSIS_HOOK_POSTINSTALL
  DetailPrint "$(smtc2_registering_path)"
  Push "$INSTDIR"
  ; Expanded here (inside the install section), so ${INSTALLMODE} is defined.
  !if "${INSTALLMODE}" == "currentUser"
    Call SMTC_AddToPathHKCU
  !else
    Call SMTC_AddToPathHKLM
  !endif
  ; Broadcast with a timeout; a blocking SendMessage hangs setup if any
  ; top-level window fails to pump the message.
  SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=1000
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; Uses built-in checkbox on uninstall confirm page
  ; $DeleteAppDataCheckboxState is set by un.ConfirmLeave
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  DetailPrint "$(smtc2_removing_path)"
  Push "$INSTDIR"
  !if "${INSTALLMODE}" == "currentUser"
    Call un.SMTC_RemoveFromPathHKCU
  !else
    Call un.SMTC_RemoveFromPathHKLM
  !endif
  SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=1000

  ${If} $DeleteAppDataCheckboxState = 1
  ${AndIf} $UpdateMode <> 1
    DetailPrint "$(smtc2_deleting_data)"
    RMDir /r "$APPDATA\smtc2web"
    RMDir /r "$LOCALAPPDATA\smtc2web"
  ${EndIf}
!macroend
