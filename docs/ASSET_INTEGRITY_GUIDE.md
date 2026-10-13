# 🎨 Asset Integrity & Zero-External-Image Guide

> **규정 선언: 모든 프로젝트 자산(SVG, PNG, GIF, 다이어그램)의 100% 로컬 영구 보존 및 GitHub 푸시 전 시각적 렌더링 캡처 검증 게이트 표준**

---

## 📌 2대 절대 원칙 (Two Uncompromising Invariants)

### 1. 외부 이미지 절대 금지 및 100% 로컬 자체 저장 (Zero External Image Dependency)
- **배경 및 위험성**:
  - 외부 이미지 호스팅(Imgur, Postimages, 블로그, 외부 S3 등)을 마크다운 이미지 링크로 사용하면, 해당 외부 서비스의 서비스 종료, 트래픽 차단, 계정 정지, 링크 변경으로 인해 프로젝트 문서에 치명적인 **404 이미지 깨짐(엑스박스)** 현상이 발생합니다.
- **철칙**:
  - 프로젝트에 사용되는 모든 다이어그램(SVG), 아키텍처 구조도, 시연 애니메이션(GIF), 스크린샷(PNG)은 **100% 프로젝트 내부(`docs/assets/...`)에 직접 파일로 커밋/저장**해야 합니다.
  - 마크다운 문서에서는 오직 저장소 상대 경로(`./docs/assets/...` 또는 `../assets/...`)나 본인 리포지토리의 원본 경로만 사용합니다.
  - 예외: 상태 표시용 표준 SVG 배지(`img.shields.io`)는 허용되나, 주요 본문 시각화 이미지는 전량 자체 호스팅을 강제합니다.

---

### 2. GitHub 반영 전 시각적 렌더링 캡처 검증 게이트 (Visual Render Capture Gate)
- **배경 및 위험성**:
  - SVG 파일 내에 XML 예약어(`&`, `<`, `>`, `"`)가 엔티티(`&amp;`, `&lt;` 등)로 이스케이프되지 않은 채 생으로 들어가면, 브라우저의 XML 파서가 `XML Parsing Error`를 발생시켜 화면에 이미지가 전혀 렌더링되지 않고 깨진 이미지 아이콘이 출력됩니다.
  - 또한, SVG `<text>` 요소의 텍스트가 박스 경계를 넘치거나(overflow) 글꼴이 겹치는 레이아웃 결함이 발생할 수 있습니다.
- **철칙**:
  - GitHub에 커밋 및 푸시하기 전, **반드시 실제 브라우저(Puppeteer / Headless Chromium)를 통해 렌더링하고 스크린샷 캡처를 수행하여 시각적 무결성을 인간 또는 AI가 직접 눈으로 확인**해야 합니다.
  - 모든 텍스트는 박스 내부에서 안전하게 줄바꿈(`formatSvgDesc` / `tspan`) 처리되어야 하며, 박스 밖으로 1픽셀도 넘치지 않아야 합니다.

---

## 🛠️ 자동 검증 러너 (`scripts/verify_assets.mjs`)

본 툴킷에 포함된 `scripts/verify_assets.mjs`를 실행하면 아래 3단계 검증이 전자동으로 수행됩니다:

```powershell
# 1. 일반 검증 (외부 이미지 누출 검사 + SVG XML 문법 정밀 감사)
node scripts/verify_assets.mjs

# 2. 브라우저 렌더링 스크린샷 자동 캡처 검증 (Puppeteer 헤드리스 구동)
node scripts/verify_assets.mjs --capture
```

### 자동 수행 내역:
1. **외부 이미지 누출 전수 스캔**: 프로젝트 내 모든 `.md` 파일에서 `http://`, `https://` 외부 이미지 참조를 탐지하여 경고 및 차단.
2. **SVG XML 엔티티 파서 검사**: 생 `&` 문자, 태그 열림/닫힘 불일치 등 파싱 오류 원천 적발.
3. **Headless 브라우저 렌더링 캡처**: 실제 렌더링 결과물을 `docs/render_verification_captured.png`로 캡처하여 저장.
