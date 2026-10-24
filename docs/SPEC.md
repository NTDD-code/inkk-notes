# INKK MATHSTUDIO: ĐẶC TẢ NGÔN NGỮ LỆNH (v1)

## 0. LUẬT TỐI THƯỢNG
1. Spec này là nguồn sự thật duy nhất. Cái gì không có trong spec = KHÔNG hỗ trợ.
   Không được tự đoán cú pháp, không được "làm gần giống".
2. Gặp lệnh/cú pháp ngoài spec -> báo lỗi theo mục 9. TUYỆT ĐỐI không vẽ hay in kết quả bừa.
3. Chỗ nào spec nêu thì hành vi phải giống MATLAB thật. Chỗ lệch MATLAB được đánh dấu [KHÁC MATLAB].
4. Không sửa phần ngoài MathStudio (whiteboard, ink, resize 8 chốt...) trừ khi bắt buộc để gắn plot.
5. Mỗi lần chỉ làm MỘT tier. Xong phải chạy `cargo test` và báo số test pass/fail.
   Không được báo "xong" khi chưa có test chạy qua.

## 1. KIẾN TRÚC BẮT BUỘC
- Pipeline: Lexer -> Parser (ra AST) -> Evaluator. CẤM dùng regex / split chuỗi / so khớp chuỗi để "hiểu" lệnh.
- Module interpreter thuần Rust, không phụ thuộc UI, để test được.
- Value chỉ có 2 loại:
    Matrix { rows, cols, data: Vec<f64> }   (lưu column-major; số vô hướng = ma trận 1x1)
    Str(String)
  Boolean là 0/1 (kiểu Matrix).
- Hàm built-in đăng ký trong một registry: tên -> { min_args, max_args, fn }.
  Evaluator kiểm tra số đối số TRƯỚC khi gọi hàm.
- Evaluator KHÔNG vẽ trực tiếp. Nó trả về danh sách PlotCommand / trạng thái Figure, UI mới render.
- Preset là script bình thường chạy qua cùng interpreter. Cấm hardcode kết quả.
- Không dùng unwrap()/panic trong evaluator. Mọi lỗi trả về Result<_, MathError>.

## 2. CÚ PHÁP
- Câu lệnh kết thúc bằng xuống dòng hoặc `;` hoặc `,`.
  Có `;` -> không in kết quả. Không có -> in `tên = giá trị`.
- Biểu thức không gán -> gán vào `ans` và in `ans = ...`.
- Comment: từ `%` đến hết dòng. Nối dòng: `...` ở cuối dòng.
- Chuỗi: "abc" hoặc 'abc'. Dấu `'` là transpose nếu đứng ngay sau
  tên biến, số, `)`, `]`, `'` hoặc `.'`; ngược lại là mở chuỗi.
- Tên biến: [A-Za-z_][A-Za-z0-9_]*, phân biệt hoa/thường.
- Hằng: pi, e, Inf, NaN, eps. (KHÔNG có i, j, số phức.)
- Chỉ số bắt đầu từ 1.

### Toán tử (ưu tiên từ cao xuống thấp)
1. `'` (transpose), `.^`, `^`
2. `+x`, `-x` (một ngôi)       -> -2^2 = -4 ; 2^-1 = 0.5
3. `*`  `/`  `.*`  `./`  `\`
4. `+`  `-`
5. `:` (range)

- `*` là nhân ma trận: số cột trái phải = số hàng phải, nếu không báo lỗi.
- `.*` `./` `.^` là từng phần tử; hai ma trận phải CÙNG kích thước, hoặc một bên là số vô hướng.
- Không có broadcasting kiểu vector hàng + vector cột [KHÁC MATLAB: báo lỗi].
- `A/s` chỉ hỗ trợ khi s là vô hướng. `A\b` hỗ trợ cho A vuông.
- `A^n` chỉ hỗ trợ n là số nguyên và A vuông.

### Ma trận literal
    [1 2 3]        hàng
    [1 2; 3 4]     2x2   (`;` hoặc xuống dòng = sang hàng mới)
    [1, 2, 3]      dấu phẩy cũng được
    [A, B]  [A; B] nối ma trận, kích thước phải khớp
- Trong `[]`: khoảng trắng + dấu +/- + KHÔNG có khoảng trắng sau -> là phần tử mới.
    [1 -2] = 2 phần tử (1 và -2)     [1 - 2] = 1 phần tử (-1)     [1 -2 + 3] = [1 1]

### Range và chỉ số
    a:b       a:step:b      (kể cả step âm)       linspace(a,b,n)
    A(i,j)    A(i)    A(:,j)    A(i,:)    A(a:b)    x(end)    A(end,1)
    A(i,j) = v    x(k) = v   (gán vào ô; nếu vượt kích thước thì tự mở rộng, phần thừa = 0)
- `end` chỉ hợp lệ BÊN TRONG dấu ngoặc chỉ số.
- Chỉ số phải là số nguyên dương, nếu không báo lỗi.

### Lệnh dạng không ngoặc (command syntax)
CHỈ áp dụng cho danh sách này, không thêm: hold, grid, clear, clc, close, figure, who, whos
    hold on     grid off     clear x y     clc

## 3. HÀM BUILT-IN: TIER 1 (làm trước)
Tạo: zeros(n) zeros(r,c) ones(...) eye(n) linspace(a,b,n) [n mặc định 100] diag(v) rand(...)
Kích thước: size(A) [trả [r c]]  length(v)  numel(A)  isempty(A)
Lượng giác & toán: sin cos tan asin acos atan atan2(y,x) sinh cosh tanh exp log log10 log2 sqrt abs
  floor ceil round fix sign mod(a,b) rem(a,b) max(a,b) min(a,b)
  -> các hàm trên áp dụng TỪNG PHẦN TỬ.
  -> sqrt/log/asin/acos ra ngoài miền thực => báo lỗi "Complex results are not supported."
     [KHÁC MATLAB: không trả số phức, không trả NaN thay thế]. 1/0 = Inf, 0/0 = NaN, log(0) = -Inf là ĐÚNG.
Thống kê (vector -> 1 số; ma trận -> vector hàng theo từng cột):
  sum prod mean max min cumsum sort (sort tăng dần) norm(v) (chuẩn 2 cho vector)
Đại số tuyến tính: det inv trace rank transpose A\b   (A phải vuông, trừ rank)
  inv/`\` với ma trận suy biến -> lỗi "Matrix is singular to working precision." [KHÁC MATLAB: lỗi chứ không ra Inf]
In ấn: disp(x)

## 4. VẼ ĐỒ THỊ (Tier 1)
    plot(x, y)                       x,y là vector cùng độ dài
    plot(y)                          x = 1:length(y)
    plot(x, y, "r--")                LineSpec: màu r g b k m c y ; nét - -- : -. ; marker o + * . x
    plot(x, y, "DisplayName", "tên") đặt nhãn cho legend
- [KHÁC preset cũ] Đối số thứ 3 là LineSpec chuẩn MATLAB, KHÔNG phải nhãn.
  Preset cũ `plot(x,y,"Damped Wave")` phải sửa thành `plot(x,y,"DisplayName","Damped Wave")`.
  Nếu đối số thứ 3 không phải LineSpec hợp lệ và không phải cặp name-value -> báo lỗi, không đoán.
- Phụ trợ: title("..")  xlabel("..")  ylabel("..")  legend("a","b",...)  grid on|off
           xlim([a b])  ylim([a b])  hold on|off  clf  figure
- Không có `hold on` -> plot() THAY THẾ đường cũ của axes hiện tại. Có `hold on` -> THÊM vào.
- Bảng màu mặc định theo thứ tự: #0072BD #D95319 #EDB120 #7E2F8E #77AC30 #4DBEEE #A2142F (quay vòng).
- Điểm NaN/Inf KHÔNG được vẽ và làm đứt nét. Quá 20000 điểm thì downsample, không được treo UI.
- Trục: tự chọn khoảng theo dữ liệu, vạch chia "đẹp" (1, 2, 5 x 10^k).
- Mỗi figure sinh ra là MỘT đối tượng trên bảng trắng (giữ nguyên cơ chế resize/di chuyển hiện có).

## 5. ĐỊNH DẠNG IN KẾT QUẢ
    x = 5            ->  x = 5
    y = 2.5          ->  y = 2.5000        (số không nguyên: 4 chữ số thập phân)
    z = 123456.789   ->  z = 1.2346e+05    (|v| >= 1e5 hoặc 0 < |v| < 1e-4: dạng khoa học)
    s = "hi"         ->  s = hi
    A = [1 2;3 4]    ->  A =
                              1     2
                              3     4
- Ma trận: căn phải theo cột, cách nhau tối thiểu 3 khoảng trắng.
  Nếu có phần tử không nguyên thì cả ma trận dùng 4 chữ số thập phân.
- Vector rỗng in `x = [](0x0)`.

## 6. WORKSPACE
- Biến tồn tại giữa các lần Run cho đến khi `clear` hoặc đóng panel.
- `clear` xoá tất cả, `clear x y` xoá biến chỉ định, `clc` xoá Command Window.
- `who` / `whos` liệt kê biến. Tab Workspace luôn phản ánh đúng trạng thái này (tên, size, class, preview).
- Lỗi ở dòng N: các biến đã gán ở dòng 1..N-1 VẪN được giữ.

## 7. TIER 2 (CHỈ làm khi tôi yêu cầu riêng)
- Toán tử so sánh/logic: == ~= < > <= >= & | ~
- Điều khiển: for k = a:b ... end ; while ... end ; if / elseif / else / end
- Hàm ẩn danh: f = @(x) x.^2 - 1 ; gọi f(3) ; fplot(f, [a b])
- Nhiều giá trị trả về: [r, c] = size(A)
- eig (chỉ trường hợp trị riêng thực), fprintf cơ bản (%d %f %g %s \n), plot nhiều cặp: plot(x1,y1,x2,y2)

## 8. KHÔNG HỖ TRỢ (gặp là báo lỗi, KHÔNG được xử lý gần đúng)
Số phức (i, j, 1i, real/imag/conj), cell array {}, struct và truy cập `.field`,
định nghĩa `function`, class, eval, system, `!`, toán tử ++ += &&, ||,
đọc/ghi file (load save fopen readtable), symbolic (syms diff int solve), đồ thị 3D (plot3 surf mesh),
subplot, mọi toolbox, mọi hàm không nằm trong mục 3/4/7.

## 9. THÔNG BÁO LỖI (dùng ĐÚNG các chuỗi này, thêm tiền tố vị trí)
Định dạng: `Error (line N): <thông báo>`, in màu đỏ, DỪNG tại lỗi đầu tiên, không crash.
- Parse error: Unexpected token ')' at line N, col M.
- Unrecognized function or variable 'foo'.
- Feature not supported in Inkk MathStudio: <tên>.
- Inner matrix dimensions must agree (2x3 * 2x3).
- Arrays have incompatible sizes for this operation (1x3 vs 1x2).
- Index exceeds array bounds (requested 5, max 3).
- Index must be a positive integer.
- Matrix must be square.
- Matrix is singular to working precision.
- Complex results are not supported.
- Not enough input arguments for 'f'. / Too many input arguments for 'f'.
- Vectors must be the same length.
- Invalid LineSpec '<chuỗi>'.

## 10. TEST BẮT BUỘC (viết thành #[test], phải pass hết trước khi báo xong Tier 1)
    2+3*4                  -> 14
    -2^2                   -> -4
    2^-1                   -> 0.5000
    [1 -2]                 -> 1x2 gồm 1, -2
    [1 - 2]                -> -1
    A=[1 2;3 4]; A*A       -> [7 10; 15 22]
    A.*A                   -> [1 4; 9 16]
    A'                     -> [1 3; 2 4]
    det(A)                 -> -2
    inv(A)                 -> [-2 1; 1.5 -0.5]
    A\[5;6]                -> [-4; 4.5]
    x=1:5; sum(x)          -> 15
    x(end)                 -> 5
    A(2,1)                 -> 3
    A(:,2)                 -> [2; 4]
    linspace(0,1,5)        -> [0 0.25 0.5 0.75 1]
    sin(pi)                -> 1.2246e-16
    1/0                    -> Inf
    [1 2 3]*[4 5 6]        -> Error: Inner matrix dimensions must agree (1x3 * 1x3).
    foo(1)                 -> Error: Unrecognized function or variable 'foo'.
    sqrt(-1)               -> Error: Complex results are not supported.
    plot(1:3, [1 2])       -> Error: Vectors must be the same length.
    syms x                 -> Error: Feature not supported in Inkk MathStudio: syms.
    x=5; y=x+;             -> Parse error, và x VẪN còn trong workspace
    plot(x,y); hold on; plot(x,z) -> figure có 2 đường, màu #0072BD rồi #D95319

## YÊU CẦU CHO LẦN LÀM NÀY
Chỉ làm TIER 1 (mục 1-6, 8-10). Không động vào Tier 2.
Viết lại preset cho đúng spec. Chạy `cargo test`, dán kết quả test ra cho tôi,
liệt kê rõ mục nào trong spec CHƯA làm được thay vì làm tạm cho có.


# PHỤ LỤC v1.1: bổ sung vào spec v1 (mọi luật mục 0 của v1 vẫn áp dụng)

## A. THỨ TỰ LÀM (bắt buộc)
Tier 1 -> Tier 1B -> Tier 2 -> Tier 3. Mỗi tier phải pass hết test của nó mới sang tier sau.
Không làm gộp, không làm trước.

## B. YÊU CẦU KIẾN TRÚC NGAY TỪ TIER 1 (để Tier 3 gắn vào được, không phải viết lại)
- Registry hàm lưu enum FunctionKind { Builtin(fn), User(ast, meta) }. Tier 1 chỉ dùng Builtin.
- Evaluator dùng CALL STACK gồm các frame; mỗi frame có scope biến RIÊNG.
  Workspace người dùng là frame gốc. (Tier 1 chỉ có 1 frame, nhưng cấu trúc phải sẵn.)
- Có bộ đếm bước thực thi và độ sâu gọi hàm ngay từ đầu; vượt giới hạn trả lỗi, KHÔNG được panic
  hay tràn stack của Rust.
- Mỗi hàm trong registry có metadata: name, signature (số đầu vào/ra), help (1-2 dòng), examples.
  Cung cấp API describe_function(name) trả metadata này (dùng cho `help` và sau này cho AI).

## C. TIER 1B: HÀM BỔ SUNG
Số học / THCS:
  gcd(a,b)  lcm(a,b)  factorial(n)  nchoosek(n,k)  isprime(n)  primes(n)  factor(n)
  nthroot(x,n)   (x âm và n lẻ thì ra số thực: nthroot(-8,3) = -2)
  gcd lcm factorial nchoosek isprime primes factor chỉ nhận số nguyên không âm,
  sai thì báo lỗi, không làm tròn ngầm.
Góc tính theo độ: sind cosd tand deg2rad rad2deg
  [sind(180)=0, cosd(90)=0 phải ĐÚNG tuyệt đối, không được ra 1.2246e-16]
Thống kê: std var median  (std/var dùng mẫu, chia n-1, giống MATLAB)  cumprod diff
Đa thức: polyfit(x,y,n)  polyval(p,x)  roots(p)
  roots chỉ trả nghiệm thực, vector cột SẮP GIẢM DẦN [KHÁC MATLAB: để kết quả xác định].
  Có nghiệm phức -> lỗi "Complex results are not supported."
Giải tích số: trapz(x,y)
Vector 3D: dot(a,b)  cross(a,b)  (cross chỉ nhận vector 3 phần tử)
Ma trận: reshape(A,r,c)  fliplr  flipud  A(:) (cột hoá, column-major, giống MATLAB)
Ngẫu nhiên: rng(seed)  randi(n)  randi(n,r,c)  rand/randn có thể tái lập bằng rng.
Chuỗi: num2str(x)   quy tắc: số nguyên -> không thập phân; còn lại dùng %.{N}g
  với N = max(floor(log10(|x|)) + 5, 5), tối đa 16.
  Ghép chuỗi bằng ["abc " num2str(3)] chỉ khi MỌI phần tử là chuỗi
  [KHÁC MATLAB: trộn chuỗi với số thì báo lỗi].
Định dạng hiển thị: format short | long | rat
  `format rat` hiển thị số dưới dạng phân số khi có thể: 0.75 -> 3/4 ; 1/3 -> 1/3.

Đồ thị bổ sung:
  bar(y)  bar(x,y)  hist(v)  hist(v,nbins) [mặc định 10 cột]  pie(v)  scatter(x,y)
  xline(v)  yline(v)  (đường thẳng đứng/ngang, dùng cho tiệm cận)
  axis equal  (tỉ lệ trục x:y = 1:1, cần cho đường tròn)
  axis([xmin xmax ymin ymax])
  text(x,y,"nội dung")
- Các loại này tuân theo luật hold on/off như plot. pie KHÔNG được trộn với loại khác:
  lỗi "Pie charts cannot be combined with other plots."

Thêm vào danh sách "command syntax": axis, format, help, which.

Lỗi mới (dùng đúng chuỗi):
- Arguments must be integers.
- Input must be a nonnegative integer.
- k must not exceed n.
- cross requires 3-element vectors.

## D. TEST TIER 1B (so sánh số thực với sai số tuyệt đối 1e-9)
    gcd(12,18)                    -> 6
    lcm(4,6)                      -> 12
    factorial(5)                  -> 120
    nchoosek(5,2)                 -> 10
    primes(20)                    -> [2 3 5 7 11 13 17 19]
    factor(60)                    -> [2 2 3 5]
    isprime(7)                    -> 1
    nthroot(-8,3)                 -> -2
    sind(30)                      -> 0.5000
    cosd(90)                      -> 0   (đúng tuyệt đối)
    mean([1 2 3 4])               -> 2.5000
    median([3 1 2])               -> 2
    std([1 2 3 4])                -> 1.2910
    var([1 2 3 4])                -> 1.6667
    cumprod([1 2 3 4])            -> [1 2 6 24]
    diff([1 4 9 16])              -> [3 5 7]
    polyfit([1 2 3],[2 4 6],1)    -> [2 0]
    polyval([1 0 -1],3)           -> 8
    roots([1 -3 2])               -> [2; 1]
    roots([1 0 1])                -> Error: Complex results are not supported.
    trapz([0 1 2],[0 1 4])        -> 3
    dot([1 2 3],[4 5 6])          -> 32
    cross([1 0 0],[0 1 0])        -> [0 0 1]
    reshape(1:6,2,3)              -> [1 3 5; 2 4 6]
    A=[1 2;3 4]; A(:)             -> [1;3;2;4]
    nchoosek(3,5)                 -> Error: k must not exceed n.
    factorial(-1)                 -> Error: Input must be a nonnegative integer.
    gcd(1.5,3)                    -> Error: Arguments must be integers.
    num2str(pi)                   -> 3.1416
    num2str(5)                    -> 5
    format rat; 0.75              -> ans = 3/4
    rng(1); a=rand(); rng(1); b=rand(); a==b  -> 1 (tái lập được)
    bar([1 2 3])                  -> Figure có 1 series loại Bar, 3 cột
    plot(1:3,1:3); hold on; pie([1 2])  -> Error: Pie charts cannot be combined with other plots.

## E. TIER 3: HÀM NGƯỜI DÙNG VÀ THƯ VIỆN (chỉ làm khi được yêu cầu; cần Tier 2 xong)

### E1. Định nghĩa hàm: dùng chính ngôn ngữ MathStudio
    function y = tenham(a, b)
        % Dòng help đầu tiên: mô tả ngắn.
        % Các dòng comment liên tiếp ngay sau dòng function là nội dung `help tenham`.
        y = a + b;
    end
- Dạng được hỗ trợ: `function y = f(a,b)`, `function [u,v] = f(a)`, `function f(a)` (không trả giá trị).
- Mỗi file có đúng MỘT hàm công khai, tên file = tên hàm (vd sum2.m). Cho phép thêm hàm phụ
  (local function) bên dưới trong cùng file, bên ngoài KHÔNG gọi được.
- Biến hoàn toàn cục bộ: không đọc/ghi workspace người gọi, KHÔNG có global.
- Hỗ trợ: nargin, nargout, return, error("thông báo"), đệ quy.
- Biến đầu ra phải được gán trước khi kết thúc, nếu không:
  Error: Output argument 'y' was not assigned during the call to 'f'.
- Hàm người dùng được dùng trong hàm ẩn danh và fplot như hàm built-in.
- `clear`, `clc` trong hàm chỉ tác động frame của hàm. plot/hold/disp trong hàm được phép
  (tác động lên figure chung).

### E2. Thư viện (library)
    library/
      _local/            hàm lẻ của người dùng
        sum2.m
      <tenthuvien>/
        manifest.toml    name, version, author, description, requires_inkk
        ham1.m  ham2.m
        examples/*.m     (tùy chọn)
- Giao diện: panel "Thư viện" có Add (chọn thư mục hoặc gói .zip), Remove, Enable/Disable,
  xem danh sách hàm và help từng hàm.
- Lệnh: `help tenham` in help; `which tenham` cho biết hàm thuộc built-in hay thư viện nào.
- Khi nạp, parse và kiểm tra TOÀN BỘ file. File nào lỗi (parse hoặc dùng tính năng ngoài spec, mục 8 của v1)
  thì từ chối RIÊNG hàm đó, báo `tên_file:dòng`, các hàm còn lại vẫn nạp.
- Quy tắc tên: [A-Za-z][A-Za-z0-9_]*, không trùng built-in, không trùng từ khoá.
  Trùng built-in -> Error: Cannot define 'sin': name is a built-in function.
  Trùng giữa hai thư viện -> Error: Function 'f' is defined in both 'libA' and 'libB'.
  (không ghi đè ngầm; người dùng phải Disable một bên.)

### E3. An toàn (không có ngoại lệ cho thư viện)
- Hàm người dùng bị ràng buộc y hệt script: không đọc/ghi file, không system, không mạng (mục 8 của v1).
- Giới hạn mặc định mỗi lần Run: 10.000.000 bước, độ sâu gọi hàm 200, thời gian 5 giây,
  ma trận tối đa 10.000.000 phần tử. Vượt -> lỗi, UI KHÔNG đơ, có nút Stop.
  Error: Maximum recursion depth exceeded.
  Error: Execution limit exceeded.
- Giới hạn lưu thành cấu hình, không hardcode rải rác.

### E4. Chia sẻ bảng trắng
- File bảng trắng NHÚNG mã nguồn các hàm người dùng mà script/đối tượng trên bảng đã dùng
  (kèm tên thư viện, version, hash).
- Khi mở trên máy khác: hiện thông báo "Bảng này chứa hàm tùy chỉnh: ...". Bản nhúng chỉ có hiệu lực
  trong phạm vi bảng đó, KHÔNG ghi đè thư viện cục bộ.

### E5. TEST TIER 3
    sum2.m: function s = sum2(a,b); s = a + b; end         sum2(2,3) -> 5
    f.m với `if nargin < 2; b = 10; end; y = a + b`        f(1) -> 11 ; f(1,2) -> 3
    [a,b] = rng2([3 1 2])  (hàm trả [min,max])             a=1 , b=3
    hàm không gán biến đầu ra                              -> Error: Output argument 'y' was not assigned during the call to 'f'.
    sum2(1,2,3)                                            -> Error: Too many input arguments for 'sum2'.
    file sin.m                                             -> bị từ chối khi nạp: Cannot define 'sin': name is a built-in function.
    fact(5) với fact đệ quy                                -> 120
    fact(1e6)                                              -> Error: Maximum recursion depth exceeded. (KHÔNG crash)
    function f(); while 1; end; end ; f()                  -> Error: Execution limit exceeded. (UI vẫn mượt)
    x=1; hàm g đặt x=5 bên trong; g(); x                   -> x vẫn = 1
    file có eval                                           -> từ chối: Feature not supported in Inkk MathStudio: eval (kèm dòng)
    help sum2                                              -> in đúng khối comment đầu tiên
    describe_function("sum2")                              -> trả name, signature, help