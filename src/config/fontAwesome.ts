// FontAwesome configuration
import { config, library } from "@fortawesome/fontawesome-svg-core";

// 关闭运行时自动注入 CSS。Tauri 会为 index.html 中的内联样式添加 CSP nonce，
// 浏览器据此忽略 style-src 的 'unsafe-inline'，导致 FontAwesome 在运行时插入的
// <style> 被拦截，图标因此失去尺寸（过大）或不可见。样式改为在 main.ts 中静态导入。
config.autoAddCss = false;
import {
  faMusic,
  faPaintBrush,
  faGear,
  faImage,
  faHourglass,
  faUpload,
  faTrash,
  faCheckCircle,
  faTimes,
  faExclamationCircle,
  faExclamationTriangle,
  faInfoCircle,
  faSpinner,
  faCheck,
  faFloppyDisk,
  faDownload,
  faRotate,
  faCircleUp,
  faCodeBranch,
  faCopy,
  faStore,
  faTag,
  faExternalLink,
  faGlobe,
  faBook,
  faCircleQuestion,
} from "@fortawesome/free-solid-svg-icons";

// Add icons to the library
library.add(
  faMusic,
  faPaintBrush,
  faGear,
  faImage,
  faHourglass,
  faUpload,
  faTrash,
  faCheckCircle,
  faTimes,
  faExclamationCircle,
  faExclamationTriangle,
  faInfoCircle,
  faSpinner,
  faCheck,
  faFloppyDisk,
  faDownload,
  faRotate,
  faCircleUp,
  faCodeBranch,
  faCopy,
  faStore,
  faTag,
  faExternalLink,
  faGlobe,
  faBook,
  faCircleQuestion,
);
