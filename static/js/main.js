// static/js/main.js

// 模拟的目录结构数据（在真实应用中，这应该是一个从后端获取的 JSON 数据）
const MOCK_FILE_DATA = [
  { name: "Documents", type: "dir", size: null, modified: "2025-01-01" },
  { name: "Images", type: "dir", size: null, modified: "2025-07-04" },
  { name: "setup.exe", type: "file", size: 1024576, modified: "2025-05-20" },
  { name: "README.md", type: "file", size: 4500, modified: "2025-10-06" },
  { name: "Code", type: "dir", size: null, modified: "2025-09-15" },
];

document.addEventListener("DOMContentLoaded", () => {
  const fileList = document.getElementById("file-list");
  fileList.innerHTML = ""; // 清除占位符

  // 假设从后端获取数据（这里使用模拟数据）
  const data = MOCK_FILE_DATA;

  data.forEach((item) => {
    const listItem = document.createElement("li");

    // 根据类型设置图标和样式
    const iconClass = item.type === "dir" ? "dir-icon" : "file-icon";
    const iconSymbol = item.type === "dir" ? "📁" : "📄";

    // 格式化文件大小
    const sizeDisplay = item.type === "file" ? formatBytes(item.size) : "---";

    listItem.innerHTML = `
            <span class="icon ${iconClass}">${iconSymbol}</span>
            <span class="name" style="flex-grow: 1;">${item.name}</span>
            <span class="size" style="width: 100px; text-align: right; color: #777;">${sizeDisplay}</span>
            <span class="modified" style="width: 120px; text-align: right; color: #999;">${item.modified}</span>
        `;

    fileList.appendChild(listItem);
  });
});

/** 格式化字节大小 */
function formatBytes(bytes, decimals = 2) {
  if (bytes === 0) return "0 Bytes";
  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ["Bytes", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + " " + sizes[i];
}
