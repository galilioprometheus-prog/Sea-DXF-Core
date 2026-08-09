# SeaCad

[English](README.md) | **Tiếng Việt**

SeaCad là CAD kernel clean-room được viết bằng Rust. Mục tiêu phát hành đầu tiên
là core DXF lossless, có giới hạn tài nguyên rõ ràng, hỗ trợ DXF ASCII và Binary
từ AC1009 đến AC1032.

SeaCad hiện là phần mềm pre-1.0. Mã nguồn đang được hiển thị theo giấy phép độc
quyền; việc nhìn thấy mã nguồn không đồng nghĩa được phép sử dụng, sao chép,
sửa đổi hoặc phân phối lại dự án.

## Vì sao có SeaCad

File CAD kết hợp dữ liệu kỹ thuật tồn tại lâu dài với cú pháp theo phiên bản,
payload mở rộng và các tham chiếu không được phép suy đoán. Vì vậy core SeaCad
tuân theo bốn nguyên tắc:

- bảo toàn byte gốc và provenance;
- giữ payload chưa biết hoặc độc quyền ở dạng opaque thay vì loại bỏ;
- từ chối edit mơ hồ hoặc không hợp lệ thay vì âm thầm sửa input;
- ghi sang đích mới và mở lại độc lập để kiểm chứng kết quả.

Đây là implementation native. Autodesk và các sản phẩm CAD khác có thể được
dùng như behavioral oracle cô lập, không bao giờ là runtime dependency của
parser hoặc nguồn mã để sao chép implementation.

## Trạng thái hiện tại

Quá trình phát triển đã hoàn tất đến checkpoint **M14.4v**. Core hiện tại có
thể:

- frame và mở DXF ASCII/Binary AC1009-AC1032 trong giới hạn tài nguyên;
- giữ source identity bất biến và provenance chính xác của từng record;
- cung cấp semantics HEADER, table, block, object, entity, handle, text và
  geometry đã được review mà không nâng cấp các field chưa review;
- bảo toàn dữ liệu unknown, custom, proxy và application-defined dưới dạng raw
  evidence;
- lập kế hoạch edit bất biến với conflict check và inverse plan khôi phục đúng
  từng byte;
- ghi kết quả preserve-patch sang path mới, strict reparse và kiểm chứng typed
  postcondition;
- thực hiện đường CRUD entity đã kiểm chứng đầu tiên cho record POINT canonical;
- project snapshot semantic POINT fail-closed xuyên tài liệu thành family draft
  canonical đã bind với đích, dùng binding symbol và handle cục bộ tường minh,
  nhưng chưa insertion;
- ghép ngay projection POINT chính xác đó với encoded XDATA thuộc cùng source
  entity, trong khi projection độc lập vẫn từ chối XDATA để không làm rơi payload;
- giữ provenance POINT xuyên atomic insertion, strict verification và
  create-new write journal với cleanup cùng inverse chính xác thực thi được;
- adapt clone POINT AC1032 sang AC1009 chỉ khi legacy placement và lineweight
  `BY_LAYER` tương đương có evidence, đồng thời từ chối giá trị không biểu diễn được;
- chuyển mã một text span chính xác giữa hai tài liệu DXF được parse độc lập,
  với decode/encode có giới hạn và không thay thế ký tự, rồi kiểm chứng
  round-trip Unicode đúng từng byte trước khi cung cấp byte đích;
- áp dụng chuyển mã đã kiểm chứng cho XDATA string group-1000, giữ receipt gọn
  theo từng occurrence xuyên application/entity và quá trình ghi POINT clone,
  đồng thời vẫn bind APPID/LAYER theo đích thay vì tự động dịch hoặc đổi tên;
- chuyển mã color-book name group 430 của POINT với provenance chính xác từ
  source field, đồng thời giữ receipt xuyên family/XDATA draft, insertion,
  strict verification và create-new write journal;
- cung cấp ledger hoàn thiện entity đã audit, đặt POINT ở mức verified mutation
  (mức 5/6), đồng thời giữ qualification private corpus và CI sáu nền tảng tại
  checkpoint hiện tại làm blocker release tường minh;
- đánh giá point của SPLINE rational hoặc non-rational đã đạt analytic readiness
  bằng phép tính De Boor homogeneous có giới hạn, kiểm tra đúng parameter domain,
  giữ failure typed và không tự động tessellate;
- đánh giá first derivative tương ứng của SPLINE rational bằng derivative
  control polygon có giới hạn và homogeneous quotient rule, đồng thời giữ
  evaluated point cùng tangent vector trong một kết quả;
- báo cáo mức hoàn thiện curve family đã audit một cách tường minh: SPLINE ở
  geometry 4/6, HELIX ở typed semantics 3/6 và chưa entity nào được gọi complete;
- giữ raw evidence HATCH và MESH hiện đại theo đúng subclass, gồm cả subclass
  lặp và group code lồng nhau bị trùng, nhưng chưa gán role, decode topology hay
  suy diễn applicability;
- decode 25 role scalar/text HATCH không bị trùng nghĩa vào đúng wire domain,
  đồng thời vẫn giữ raw các collision thuộc boundary, seed point và pattern line;
- cung cấp 25 cardinality card cố định cho mỗi HATCH subclass chính xác, với
  member reference gọn và trạng thái absent/unique/multiple độc lập;
- chọn singleton scalar HATCH theo kiểu fail-closed, với default extrusion đã
  review, kiểm tra finite/domain và invalid evidence bám source;
- lắp ráp một tuple extrusion HATCH chính xác, không normalize cho mỗi subclass,
  đồng thời giữ provenance explicit/defaulted của từng component và outcome
  typed cho component không khả dụng hoặc vector 0;
- phân vùng mỗi HATCH subclass quanh hai fence group 91/group 75 duy nhất và
  đúng thứ tự, giữ opaque payload boundary lồng nhau và fail-closed nếu anchor
  bị thiếu, lặp hoặc đảo thứ tự;
- chọn tuple elevation HATCH bắt buộc và chính xác từ header đã cô lập, giữ
  signed zero cùng raw provenance, đồng thời từ chối nested decoy, component
  không khả dụng, giá trị non-finite và planar component khác 0;
- nhóm chính xác boundary path HATCH theo anchor group 92, giữ field orphan
  trước anchor cùng payload opaque theo từng path, rồi đối chiếu số path quan
  sát được với khai báo group 91 đã decode;
- decode các bit boundary-path group 92 đã được tài liệu hóa và phân loại path
  hợp lệ thành Polyline hoặc Edges, đồng thời từ chối số âm và bit ngoài chuẩn;
- chọn header group 72/73/93 bắt buộc cho Polyline boundary path với provenance
  chính xác cùng domain boolean/count đã review;
- nhóm vertex Polyline boundary theo group 10 với cardinality Y/bulge riêng,
  orphan trước anchor và relation declared-versus-observed vertex count;
- chọn numeric semantics X/Y/bulge hữu hạn, bám source cho các vertex Polyline
  đã nhóm, đồng thời giữ tường minh trạng thái absent, duplicate, ASCII lỗi và
  Binary non-finite mà chưa suy diễn default bulge;
- áp dụng default 0 đã được tài liệu hóa cho bulge Polyline vắng mặt và enforce
  envelope has-bulge group 72 mà không làm mất numeric evidence nền;
- lắp ráp tuple OCS X/Y bắt buộc, bám source cho Polyline vertex với signed-zero
  chính xác và mask component không khả dụng có kiểu;
- dựng topology segment Polyline open/closed theo kiểu fail-closed với endpoint
  chính xác và đối chiếu declared count, nhưng chưa suy diễn geometry;
- phân loại mỗi segment Polyline thành straight, bulge-arc hoặc indeterminate
  từ start vertex và vẫn giữ exact bulge evidence;
- cung cấp line geometry OCS với endpoint chính xác cho segment Polyline
  straight, đồng thời giữ outcome typed cho arc, shape bất định và endpoint
  không khả dụng;
- suy ra center, radius và signed sweep OCS hữu hạn cho Polyline bulge arc,
  đồng thời trả failure typed cho chord suy biến và số học dẫn xuất non-finite;
- chiếu line và arc geometry HATCH Polyline sang tọa độ WCS hữu hạn từ elevation
  chính xác cùng extrusion explicit/defaulted đã normalize;
- chọn edge count bắt buộc cho HATCH boundary path loại Edges và nhóm chính xác
  các anchor group 72 với đối chiếu declared/observed;
- decode mỗi marker HATCH edge đã nhóm thành Line, CircularArc, EllipticArc hoặc
  Spline, đồng thời giữ giá trị lỗi và ngoài domain dưới dạng issue có kiểu,
  bám source mà chưa chọn các field payload của edge;
- cung cấp bốn cardinality card theo role chính xác cho mỗi HATCH Line edge đã
  định kiểu, bao phủ field OCS X/Y đầu-cuối và giữ độc lập trạng thái
  absent/unique/multiple mà chưa decode giá trị số;
- kiểm tra và validate entity XDATA, gồm evidence APPID/LAYER chính xác ở nguồn
  và tài liệu đích được parse độc lập, trạng thái symbol/cấu trúc theo từng
  application, capacity/coordinate/handle/payload-envelope theo từng entity,
  projection giá trị logic, canonical group encoding ở đích và nhóm encoded
  chính xác theo từng application/entity, ghép vào draft record ở đích và lập
  insertion plan nguyên tử, strict post-image verification và create-new write
  journal cho XDATA, cùng kết quả staging thay thế handle
  group-1005 cùng dialect ghi sang path mới rồi strict reparse để xác minh;
- tạo schema xác định, CycloneDX evidence, legal bundle, native package và
  receipt phát hành sáu nền tảng.

Ranh giới hỗ trợ chính xác luôn hẹp hơn raw inventory. Nhận diện được tên entity
không đồng nghĩa đã hỗ trợ đầy đủ semantic hoặc edit. Xem
[support matrix](docs/SUPPORT_MATRIX.md) và
[subplan hoàn thiện DXF entity](docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md) để biết chi
tiết chuẩn tắc.

## Chưa hỗ trợ

- Chưa hoàn tất điều kiện phát hành DXF Core 1.0.
- Chưa tuyên bố typed semantics và CRUD đầy đủ cho mọi DXF entity.
- Chưa implement DWG, DGN V7 và DGN V8.
- Chưa implement rendering, constraint solving và B-rep modeling kernel native.
- Không bao giờ ghi đè trực tiếp file nguồn.
- Rust API chưa ổn định.

Roadmap dài hạn triển khai DGN V7, DGN V8 và DWG native clean-room sau DXF Core
1.0. Base plan không dùng ODA, RealDWG, Bentley SDK hoặc format runtime độc
quyền khác; evidence chưa đủ sẽ giữ capability ở opaque hoặc read-only thay vì
suy đoán parser hay writer.

## Workspace

| Crate | Trách nhiệm |
|---|---|
| `seacad-dxf-core` | Framing DXF lossless có giới hạn, semantics/geometry đã review, lập kế hoạch edit đảo ngược và writer đã kiểm chứng |
| `seacad-cli` | Lệnh inspect/verify dạng human và JSON cùng corpus receipt đã loại thông tin nhạy cảm |
| `seacad-schema-gen` | Schema xác định, dependency/legal evidence, native package và kiểm chứng receipt |

Tài liệu quan trọng:

- [Master implementation plan](docs/IMPLEMENTATION_PLAN.md)
- [Support matrix](docs/SUPPORT_MATRIX.md)
- [Subplan implementation DXF Core 1.0](docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md)
- [Subplan hoàn thiện DXF entity](docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md)
- [Chính sách dependency](docs/DEPENDENCY_POLICY.md)
- [Chính sách toolchain và oracle](docs/TOOLCHAIN.md)
- [Hợp đồng CLI JSON v1](docs/CLI_JSON_V1.md)

Corpus CAD riêng tư không bao giờ được commit. Fixture công khai chỉ được đưa
vào repository sau khi quyền sở hữu, giấy phép, provenance và hash đã được
review.

## Build và kiểm chứng

SeaCad pin Rust 1.97.1. Chạy từ repository root:

```console
cargo +1.97.1 build --locked --workspace
cargo +1.97.1 run --locked -p seacad-cli -- inspect drawing.dxf
cargo +1.97.1 run --locked -p seacad-cli -- verify drawing.dxf
```

Human output mặc định là tiếng Anh. Dùng `--lang vi` để xuất tiếng Việt; tên
field JSON và status code luôn giữ identifier tiếng Anh ổn định.

```console
cargo +1.97.1 run --locked -p seacad-cli -- inspect drawing.dxf --lang vi
cargo +1.97.1 run --locked -p seacad-cli -- verify drawing.dxf --json
```

Các gate local bắt buộc:

```console
cargo deny --locked check
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy --locked --workspace --all-targets -- -D warnings
cargo +1.97.1 test --locked --workspace
git diff --check
```

GitHub Actions được thiết kế chạy thủ công và tiết kiệm ngân sách. Công việc
thông thường được kiểm chứng local; workflow phát hành sáu nền tảng chỉ được
dispatch tại checkpoint đã review.

## Bảo mật và xử lý dữ liệu

Mọi CAD input đều không đáng tin cậy. Production Rust cấm unsafe code và các
lối tắt dựa trên panic. Parser áp dụng giới hạn tài nguyên rõ ràng, report mặc
định ẩn path và writer tạo đích mới thay vì ghi đè source.

Không đính kèm bản vẽ bí mật hoặc bản vẽ bên thứ ba vào public issue. Hãy dùng
minimal synthetic reproducer có quyền phân phối rõ ràng.

## Đóng góp

Repository hiện chưa cho phép tự do tái sử dụng hoặc phân phối lại mã. Trước
khi đóng góp code, fixture, quan sát định dạng hoặc bảng dữ liệu dẫn xuất, hãy
mở discussion mô tả nguồn và giấy phép. Đóng góp phải giữ ranh giới clean-room,
không chứa code độc quyền sao chép, tài liệu SDK, bản vẽ khách hàng hoặc dữ liệu
từ bản phân phối phần mềm không chính thức.

## Tài trợ và giấy phép

Copyright (c) 2026 SeaCad. All rights reserved. Xem [LICENSE](LICENSE).

Việc đánh giá repository hiển thị không cấp quyền dùng trong production, sửa
đổi hoặc phân phối lại. Người muốn sử dụng SeaCad phải có thỏa thuận riêng bằng
văn bản với chủ dự án. Dự án dự định giữ chi phí tiếp cận quy mô nhỏ ở mức hợp
lý và có thể cấp giấy phép phù hợp đổi lấy một khoản tài trợ nhỏ được thỏa thuận
theo từng trường hợp.

Khoản tài trợ tự nó không cấp quyền nếu văn bản đi kèm không quy định rõ. Việc
nhúng thương mại, cung cấp hosted service, phân phối lại và bridge dùng SDK của
vendor có thể cần điều khoản khác.
