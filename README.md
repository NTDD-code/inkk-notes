<div align="center">

<img src="https://github.com/NTDD-code/inkk-notes/tree/main/assets/assets/pixel/banner.svg" alt="HandNote" width="640">

![Rust](https://img.shields.io/badge/Rust-100%25-dea584?logo=rust&logoColor=white)
![Platform](https://img.shields.io/badge/Windows%20%7C%20Linux-native-4c8bf5)
![Size](https://img.shields.io/badge/binary-~14%20MB-2ea44f)
![RAM](https://img.shields.io/badge/RAM-~25--35%20MB-orange)
![License](https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-blue)

</div>

<!-- Thêm ảnh chụp màn hình vào thư mục assets rồi bỏ comment dòng dưới -->
<!-- <p align="center"><img src="assets/screenshot.png" width="720" alt="HandNote"></p> -->

<details open>
<summary><b>🇻🇳 Tiếng Việt</b></summary>

<br>

<div align="center">

**Ứng dụng ghi chú viết tay, nhẹ và mượt, viết hoàn toàn bằng Rust.**

Chạy native trên Windows và Linux. Không WebView, không HTML, không CSS, không JavaScript.

</div>

---

## Giới thiệu

Mình làm HandNote để có một ứng dụng ghi chú viết tay gọn gàng, mở lên là viết ngay, không phải chờ một trình duyệt nhúng khởi động. Toàn bộ giao diện được vẽ trực tiếp bằng GPU thông qua `eframe` / `egui`, nên file chạy chỉ khoảng **14 MB** (so với khoảng 150 MB của một app Electron tương đương) và chỉ tốn **25 đến 35 MB RAM**.

Ngoài viết tay, HandNote còn có **Inkk MathStudio**: một môi trường tính toán và vẽ đồ thị theo cú pháp gần giống MATLAB, để mình vừa ghi chú vừa giải bài ngay trên bảng trắng.

## Điểm mình chú trọng

### ✍️ Nét viết mượt và tự nhiên
- Nối các điểm bằng **đường cong Bézier bậc 2**, lấy trung điểm làm điểm neo: $M_i = (P_i + P_{i+1}) / 2$.
- **Bộ lọc low-pass** làm mịn tọa độ để bớt rung tay.
- **Làm mịn áp lực bút** để nét có độ đậm nhạt tự nhiên.

### 📄 5 kiểu giấy
| Kiểu giấy | Mô tả |
|---|---|
| Ruled | Kẻ ngang, có lề đỏ |
| Grid | Ô ly caro |
| Dot Grid | Chấm bi |
| Cornell Notes | Chia vùng ghi chú theo phương pháp Cornell |
| Blank | Giấy trắng |

### 🧽 Hai chế độ tẩy
- **Tẩy cả nét (Stroke Eraser):** chạm vào là xóa trọn nét đó.
- **Tẩy cắt điểm (Point Eraser):** cắt nét tại điểm tẩy và chia thành hai nét nhỏ.

### ♾️ Trang giấy vô hạn
Trang tự kéo dài xuống khi bạn viết gần đáy. Bạn cũng có thể bấm nút `♾️ Kéo dài (+600px)` để thêm chỗ bất cứ lúc nào.

### 🗂️ Quản lý trang trực quan
Ngăn kéo thumbnail cho phép xem trước từng trang, nhân bản hoặc xóa trang rất nhanh.

### 💾 Lưu trữ gọn
Ghi chú được lưu thành file `.handnote`, nén gzip trực tiếp bằng `flate2`.

---

<img src="assets/pixel/pi.svg" align="right" width="72" alt="">

## 🧮 Inkk MathStudio

Mình viết script, bấm chạy và xem kết quả hoặc đồ thị ngay trong ứng dụng, không cần mở thêm phần mềm nào khác. Cú pháp bám sát MATLAB để quen tay, chỗ nào khác mình ghi rõ bên dưới.

### Hàm có sẵn

| Nhóm | Hàm |
|---|---|
| Số học | `gcd` `lcm` `factorial` `nchoosek` `isprime` `primes` `factor` `nthroot` |
| Góc tính theo độ | `sind` `cosd` `tand` `deg2rad` `rad2deg` |
| Thống kê | `mean` `median` `std` `var` `cumprod` `diff` |
| Đa thức | `polyfit` `polyval` `roots` |
| Giải tích số | `trapz` |
| Vector 3D | `dot` `cross` |
| Ma trận | `reshape` `fliplr` `flipud` `A(:)` |
| Ngẫu nhiên | `rng` `rand` `randn` `randi` (tái lập được bằng `rng(seed)`) |
| Chuỗi và hiển thị | `num2str` `format short` `format long` `format rat` |

Một vài điểm mình làm kỹ:
- `sind(180)` và `cosd(90)` ra **đúng 0**, không bị lỗi làm tròn kiểu `1.2246e-16`.
- `nthroot(-8, 3)` ra `-2` (căn bậc lẻ của số âm vẫn là số thực).
- `std` và `var` chia cho `n-1`, giống MATLAB.
- `format rat` hiển thị phân số khi có thể, ví dụ `0.75` thành `3/4`.
- Các hàm số nguyên như `gcd`, `factorial` báo lỗi khi đầu vào sai, không làm tròn ngầm.

### Đồ thị

`plot` `bar` `hist` `pie` `scatter` `xline` `yline` `text` `axis equal` `axis([xmin xmax ymin ymax])`

Các loại đồ thị tuân theo luật `hold on` / `hold off`. Riêng `pie` không trộn được với loại khác. `xline` và `yline` tiện để vẽ đường tiệm cận, còn `axis equal` giúp đường tròn thật sự tròn.

### Khác MATLAB ở đâu

Mình cố ý chọn hành vi rõ ràng hơn ở vài chỗ:
- `roots` chỉ trả **nghiệm thực**, là vector cột **sắp giảm dần** để kết quả luôn xác định. Nếu có nghiệm phức, chương trình báo `Complex results are not supported.`
- Ghép chuỗi bằng `["abc " num2str(3)]` chỉ hợp lệ khi mọi phần tử đều là chuỗi. Trộn chuỗi với số sẽ báo lỗi.
- `cross` chỉ nhận vector 3 phần tử.

### Hàm người dùng và thư viện

Mình tự viết hàm bằng chính ngôn ngữ MathStudio:

```matlab
function y = sum2(a, b)
    % Cộng hai số. Dòng comment này là nội dung của `help sum2`.
    y = a + b;
end
```

- Hỗ trợ `function y = f(a,b)`, `function [u,v] = f(a)` và `function f(a)`.
- Có `nargin`, `nargout`, `return`, `error("...")` và đệ quy.
- Biến hoàn toàn cục bộ, không có `global`, hàm không đụng vào workspace của nơi gọi.
- Mỗi file `.m` có một hàm công khai, tên file trùng tên hàm. Hàm phụ trong cùng file chỉ dùng nội bộ.

Hàm được gom vào thư viện:

```
library/
├── _local/              hàm lẻ của mình
│   └── sum2.m
└── <tenthuvien>/
    ├── manifest.toml    name, version, author, description, requires_inkk
    ├── ham1.m
    └── examples/
```

Panel **Thư viện** cho phép thêm (thư mục hoặc gói `.zip`), xóa, bật tắt và xem help từng hàm. Lệnh `help tenham` in phần giới thiệu hàm, `which tenham` cho biết hàm là built-in hay thuộc thư viện nào. Nếu một file bị lỗi, chỉ riêng hàm đó bị từ chối kèm vị trí `tên_file:dòng`, các hàm còn lại vẫn nạp bình thường. Tên trùng với hàm built-in hoặc trùng giữa hai thư viện sẽ báo lỗi chứ không ghi đè ngầm.

### An toàn

Hàm người dùng bị ràng buộc y hệt script: không đọc ghi file, không gọi hệ thống, không dùng mạng. Giới hạn mặc định cho mỗi lần chạy:

| Giới hạn | Mặc định |
|---|---|
| Số bước thực thi | 10.000.000 |
| Độ sâu gọi hàm | 200 |
| Thời gian | 5 giây |
| Kích thước ma trận | 10.000.000 phần tử |

Vượt giới hạn thì báo lỗi (`Maximum recursion depth exceeded.` hoặc `Execution limit exceeded.`), giao diện không bị đơ và luôn có nút **Stop**. Các giới hạn nằm trong cấu hình, không hardcode.

### Chia sẻ bảng trắng

File bảng trắng nhúng sẵn mã nguồn các hàm người dùng mà bảng đó đã dùng (kèm tên thư viện, phiên bản và hash). Mở trên máy khác sẽ có thông báo bảng chứa hàm tùy chỉnh, và bản nhúng chỉ có hiệu lực trong bảng đó, không ghi đè thư viện cục bộ của người mở.

<!-- Cập nhật trạng thái hoàn thành từng phần (hàm cơ bản, đồ thị, hàm người dùng, thư viện) khi xong -->

---

## Bắt đầu nhanh

### Yêu cầu
- [Rust](https://rustup.rs) (bản stable mới nhất)
- Git

### Windows

```powershell
git clone https://github.com/NTDD-code/inkk-notes.git
cd inkk-notes
cargo run --release
```

Sau khi build xong, bạn cũng có thể chạy thẳng file thực thi:

```powershell
.\target\release\rust_handnote.exe
```

### Linux

Cài vài thư viện đồ họa cần thiết (ví dụ trên Ubuntu / Debian, các bản phân phối khác dùng gói tương đương):

```bash
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libasound2-dev
```

Rồi build và chạy:

```bash
git clone https://github.com/NTDD-code/inkk-notes.git
cd inkk-notes
cargo run --release
```

Bản build trên Linux có giao diện và tính năng giống hệt bản Windows.

---

## Phím tắt

| Thao tác | Phím |
|---|---|
| Hoàn tác (Undo) | `Ctrl` + `Z` |
| Làm lại (Redo) | `Ctrl` + `Y` |
| Thu phóng trang giấy | `Ctrl` + lăn chuột |
| Di chuyển khung vẽ (Pan) | Chuột giữa hoặc công cụ Pan |

---

## Cấu trúc thư mục

```
inkk-notes/
├── src/         mã nguồn chính
├── assets/      tài nguyên (ảnh, icon, pixel art)
├── docs/        tài liệu
├── packaging/   đóng gói bản phát hành
├── scripts/     script hỗ trợ
└── library/     dữ liệu cục bộ
```

<!-- Sẽ bổ sung: ảnh demo, tính năng mới, hướng dẫn đóng góp -->

---

## Giấy phép

Dự án dùng song song hai giấy phép, bạn chọn một trong hai: [MIT](LICENSE-MIT) hoặc [Apache-2.0](LICENSE-APACHE).

</details>

<details>
<summary><b>🇬🇧 English</b></summary>

<br>

<div align="center">

**A lightweight, smooth handwritten note app, written entirely in Rust.**

Native on Windows and Linux. No WebView, no HTML, no CSS, no JavaScript.

</div>

---

## About

I built HandNote because I wanted a tidy handwritten note app that is ready to write the moment it opens, with no embedded browser to wait for. The whole interface is drawn directly on the GPU through `eframe` / `egui`, so the executable is only about **14 MB** (compared with roughly 150 MB for a similar Electron app) and uses just **25 to 35 MB of RAM**.

Besides handwriting, HandNote includes **Inkk MathStudio**: a calculation and plotting environment with a MATLAB-like syntax, so I can take notes and solve problems on the same whiteboard.

## What I care about

### ✍️ Smooth, natural strokes
- Points are joined with **quadratic Bézier curves**, using midpoints as anchors: $M_i = (P_i + P_{i+1}) / 2$.
- A **low-pass filter** smooths coordinates to reduce hand tremor.
- **Pressure smoothing** gives strokes a natural thick and thin feel.

### 📄 5 paper styles
| Style | Description |
|---|---|
| Ruled | Horizontal lines with a red margin |
| Grid | Square grid |
| Dot Grid | Dotted grid |
| Cornell Notes | Sections laid out for the Cornell method |
| Blank | Plain white paper |

### 🧽 Two eraser modes
- **Stroke Eraser:** touch a stroke to remove it completely.
- **Point Eraser:** cuts a stroke at the erased point and splits it into two smaller strokes.

### ♾️ Infinite paper
The page extends downward automatically as you write near the bottom. You can also press the `♾️ Extend (+600px)` button to add room whenever you like.

### 🗂️ Visual page management
A thumbnail drawer lets you preview every page, and duplicate or delete pages quickly.

### 💾 Compact storage
Notes are saved as `.handnote` files, compressed with gzip directly through `flate2`.

---

<img src="assets/pixel/pi.svg" align="right" width="72" alt="">

## 🧮 Inkk MathStudio

I write a script, press run, and see results or plots right inside the app, with no other software needed. The syntax follows MATLAB closely so it feels familiar, and anywhere it differs, I say so below.

### Built-in functions

| Group | Functions |
|---|---|
| Arithmetic | `gcd` `lcm` `factorial` `nchoosek` `isprime` `primes` `factor` `nthroot` |
| Angles in degrees | `sind` `cosd` `tand` `deg2rad` `rad2deg` |
| Statistics | `mean` `median` `std` `var` `cumprod` `diff` |
| Polynomials | `polyfit` `polyval` `roots` |
| Numerical calculus | `trapz` |
| 3D vectors | `dot` `cross` |
| Matrices | `reshape` `fliplr` `flipud` `A(:)` |
| Random | `rng` `rand` `randn` `randi` (reproducible with `rng(seed)`) |
| Strings and display | `num2str` `format short` `format long` `format rat` |

A few details I took care over:
- `sind(180)` and `cosd(90)` return **exactly 0**, not a rounding artifact like `1.2246e-16`.
- `nthroot(-8, 3)` returns `-2` (an odd root of a negative number is still real).
- `std` and `var` divide by `n-1`, like MATLAB.
- `format rat` shows fractions when possible, for example `0.75` becomes `3/4`.
- Integer functions such as `gcd` and `factorial` raise an error on invalid input instead of rounding silently.

### Plotting

`plot` `bar` `hist` `pie` `scatter` `xline` `yline` `text` `axis equal` `axis([xmin xmax ymin ymax])`

Plot types follow the `hold on` / `hold off` rules. Only `pie` cannot be combined with other plot types. `xline` and `yline` are handy for drawing asymptotes, and `axis equal` keeps circles truly round.

### Where it differs from MATLAB

I chose clearer behavior in a few places on purpose:
- `roots` returns **real roots only**, as a column vector **sorted in descending order**, so results are always deterministic. If a complex root exists, it reports `Complex results are not supported.`
- Joining strings with `["abc " num2str(3)]` is only valid when every element is a string. Mixing strings and numbers raises an error.
- `cross` accepts only 3-element vectors.

### User functions and libraries

I write my own functions in MathStudio itself:

```matlab
function y = sum2(a, b)
    % Add two numbers. This comment is the content of `help sum2`.
    y = a + b;
end
```

- Supports `function y = f(a,b)`, `function [u,v] = f(a)` and `function f(a)`.
- Includes `nargin`, `nargout`, `return`, `error("...")` and recursion.
- Variables are fully local, there is no `global`, and a function never touches the caller's workspace.
- Each `.m` file has one public function whose name matches the file name. Helper functions in the same file stay private.

Functions are grouped into libraries:

```
library/
├── _local/              my one-off functions
│   └── sum2.m
└── <libraryname>/
    ├── manifest.toml    name, version, author, description, requires_inkk
    ├── func1.m
    └── examples/
```

The **Library** panel lets you add (a folder or a `.zip` package), remove, enable or disable, and browse the help of each function. `help name` prints a function's help text, and `which name` tells you whether it is built-in or from a library. If one file has an error, only that function is rejected, with its location as `file_name:line`, and the rest still load normally. A name that clashes with a built-in function, or with a function in another library, raises an error instead of silently overriding.

### Safety

User functions are bound by the same restrictions as scripts: no file access, no system calls, no network. Default limits per run:

| Limit | Default |
|---|---|
| Execution steps | 10,000,000 |
| Call depth | 200 |
| Time | 5 seconds |
| Matrix size | 10,000,000 elements |

Going over a limit raises an error (`Maximum recursion depth exceeded.` or `Execution limit exceeded.`), the interface never freezes, and there is always a **Stop** button. The limits live in configuration and are not hardcoded.

### Sharing whiteboards

A whiteboard file embeds the source of the user functions it uses (with library name, version and hash). Opening it on another machine shows a notice that the board contains custom functions, and the embedded copy only applies within that board, never overriding the opener's local library.

<!-- Update the status of each part (basic functions, plots, user functions, libraries) when done -->

---

## Quick start

### Requirements
- [Rust](https://rustup.rs) (latest stable)
- Git

### Windows

```powershell
git clone https://github.com/NTDD-code/inkk-notes.git
cd inkk-notes
cargo run --release
```

After building, you can also run the executable directly:

```powershell
.\target\release\rust_handnote.exe
```

### Linux

Install a few required graphics libraries (example for Ubuntu / Debian, other distributions use equivalent packages):

```bash
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libasound2-dev
```

Then build and run:

```bash
git clone https://github.com/NTDD-code/inkk-notes.git
cd inkk-notes
cargo run --release
```

The Linux build has the same interface and features as the Windows build.

---

## Keyboard shortcuts

| Action | Keys |
|---|---|
| Undo | `Ctrl` + `Z` |
| Redo | `Ctrl` + `Y` |
| Zoom the page | `Ctrl` + scroll wheel |
| Pan the canvas | Middle mouse button or the Pan tool |

---

## Project layout

```
inkk-notes/
├── src/         main source code
├── assets/      resources (images, icons, pixel art)
├── docs/        documentation
├── packaging/   release packaging
├── scripts/     helper scripts
└── library/     local data
```

<!-- To add: demo screenshots, new features, contribution guide -->

---

## License

Dual licensed, you may choose either one: [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).

</details>

---

<div align="center">

<img src="assets/pixel/squid.svg" width="72" alt="">

Nếu thấy HandNote hữu ích, một ⭐ sẽ giúp mình có thêm động lực làm tiếp.

</div>
