function n(n,e=!1){return window.__TAURI_INTERNALS__.transformCallback(n,e)}async function e(n,e={},t){return window.__TAURI_INTERNALS__.invoke(n,e,t)}var t;async function r(t,r,i){return e("plugin:event|listen",{event:t,windowLabel:i?.target,handler:n(r)}).then((n=>async()=>async function(n,t){await e("plugin:event|unlisten",{event:n,eventId:t})}(t,n)))}function i(){return e("plugin:shion-watcher|get_program_list")}function u(){return e("plugin:shion-watcher|suspend")}function a(){return e("plugin:shion-watcher|resume")}function o(n){return e("plugin:shion-watcher|is_active",{path:n})}function _(n){return r("plugin:shion-watcher://status-changed",n)}"function"==typeof SuppressedError&&SuppressedError,function(n){n.WINDOW_RESIZED="tauri://resize",n.WINDOW_MOVED="tauri://move",n.WINDOW_CLOSE_REQUESTED="tauri://close-requested",n.WINDOW_CREATED="tauri://window-created",n.WINDOW_DESTROYED="tauri://destroyed",n.WINDOW_FOCUS="tauri://focus",n.WINDOW_BLUR="tauri://blur",n.WINDOW_SCALE_FACTOR_CHANGED="tauri://scale-change",n.WINDOW_THEME_CHANGED="tauri://theme-changed",n.WINDOW_FILE_DROP="tauri://file-drop",n.WINDOW_FILE_DROP_HOVER="tauri://file-drop-hover",n.WINDOW_FILE_DROP_CANCELLED="tauri://file-drop-cancelled"}(t||(t={}));export{i as getProgramList,o as isActive,_ as onStatusChanged,a as resume,u as suspend};
