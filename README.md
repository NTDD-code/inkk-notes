# HandNote (Pure Native Rust 100%)

Ứng dụng ghi chú viết tay độc lập, đa nền tảng (**Windows & Linux**), được viết **100% bằng Rust nguyên bản**, không phụ thuộc vào WebView, HTML, CSS hay JavaScript.

---

## 🚀 Đặc điểm kỹ thuật

- **100% Pure Rust**: Sử dụng `eframe` / `egui` với GPU rendering phần cứng (OpenGL / Glow).
- **Siêu nhẹ & Hiệu năng cao**: File thực thi duy nhất chỉ **~14 MB** (thay vì 150 MB của Electron), tiêu tốn cực ít RAM (~25-35 MB).
- **Thuật toán Inking mượt mà**:
  - Đường cong Bézier bậc 2 (Quadratic Bézier Splines qua trung điểm $M_i = (P_i + P_{i+1})/2$).
  - Bộ lọc làm mịn tọa độ chống rung tay (Low-pass filter).
  - Làm mịn áp lực bút thanh đậm tự nhiên.
- **5 Kiểu giấy chuyên nghiệp**: Ruled (Kẻ ngang + lề đỏ), Grid (Ô ly caro), Dot Grid (Chấm bi), Cornell Notes, Blank (Giấy trắng).
- **Cơ chế tẩy kép (Dual Eraser)**:
  - **Tẩy cả nét (Stroke Eraser)**: Xóa trọn vẹn nét khi chạm.
  - **Tẩy cắt điểm (Point Eraser)**: Cắt đôi và chia nhỏ nét vẽ tại điểm tẩy.
- **Trang giấy vô hạn (Infinite Paper)**: Tự động kéo dài trang xuống dưới khi viết gần đáy hoặc bấm nút `♾️ Kéo dài (+600px)`.
- **Xem trước trang (Thumbnail Previews Drawer)**: Ngăn kéo xem preview trực quan từng trang, nhân bản và xóa trang nhanh.
- **Lưu trữ nén native**: File định dạng `.handnote` (chuẩn gzip nén trực tiếp bằng `flate2`).

---

## 💻 Hướng dẫn Biên dịch & Chạy

### 1. Trên Windows:
Thư mục dự án: `c:\Users\Lenovo\Downloads\TuanAnhLam_Scrble_Ink\rust_handnote`

```bash
# Chạy trực tiếp từ mã nguồn:
cargo run --release

# Hoặc chạy file .exe đã build sẵn:
.\target\release\rust_handnote.exe
```

### 2. Trên Linux (Ubuntu / Debian / Arch / Fedora):
Chỉ cần copy thư mục `rust_handnote` sang máy Linux và chạy:

```bash
# Cài đặt thư viện đồ họa cơ bản nếu chưa có (Ubuntu/Debian):
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libasound2-dev

# Build & Run:
cargo run --release
```
Sẽ tạo ra file nhị phân native chạy trực tiếp trên Linux với giao diện và tính năng giống hệt 100%!

---

## 🎨 Phím tắt
- **Ctrl + Z**: Hoàn tác (Undo)
- **Ctrl + Y**: Làm lại (Redo)
- **Ctrl + Lăn chuột**: Thu phóng trang giấy (Zoom)
- **Chuột giữa / Công cụ Pan**: Di chuyển khung vẽ (Pan)
