# ADR-0003 — 라이선스 파일 분리

- 날짜: 2026-10-06
- 상태: 확정 (Jeffrey 비준)

## 맥락

myth는 `MIT OR Apache-2.0` 듀얼 라이선스다. `rust/Cargo.toml`과 `python/pyproject.toml`의 SPDX 표기는 정확하다. 그런데 두 라이선스 전문을 `LICENSE` 파일 하나에 합쳐 두었고, 그 결과 GitHub이 라이선스를 인식하지 못한다. API의 `license.spdx_id` 값이 `NOASSERTION`이다.

여기에 문제가 두 가지 더 있었다.
- README는 라이선스를 MIT 하나로만 표기했다.
- MIT 본문에 표준과 다른 문구가 한 군데 있었다("USE OF OTHER DEALINGS"). 표준 문구는 "USE OR OTHER DEALINGS"다.

OSS 사용자에게는 라이선스가 저장소 첫 화면에서 바로 보이는 것이 중요하다.

## 선택지

- (A) 현행 유지
- (B) Rust 생태계 관례대로 `LICENSE-MIT`와 `LICENSE-APACHE`로 분리

## 선택

**(B)**

## 근거

- 듀얼 라이선스 Rust 프로젝트에서 널리 쓰이는 관례다.
- 파일을 나누면 GitHub의 라이선스 감지기가 각 전문을 인식할 수 있다.
- 패키지 메타데이터는 `license-file`이 아니라 SPDX 표현식을 쓰므로, 파일 이름을 바꿔도 빌드에는 영향이 없다.

## 영향 범위

- `LICENSE`를 삭제하고 `LICENSE-MIT`, `LICENSE-APACHE`를 추가한다.
- MIT 본문의 비표준 문구를 표준 문구로 고친다.
- `README.md` 라이선스 절에 듀얼 라이선스를 정확히 표기한다.
- `THIRD-PARTY.md`의 파일 경로 표기를 고친다.
- v1 설계 문서(`docs/03-DIRECTORY.md` 등)의 `LICENSE` 언급은 v1 기록이므로 그대로 둔다.
