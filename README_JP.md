[English](README.md) | **日本語**

# ALICE-HRM

ALICEエコシステムの人事管理 (HRM) モジュール。勤怠管理、給与計算、休暇管理、シフトスケジューリング、人事評価を純Rustで実装。

## 概要

| 項目 | 値 |
|------|-----|
| **クレート名** | `alice-hrm` |
| **バージョン** | 1.0.0 |
| **ライセンス** | AGPL-3.0 |
| **エディション** | 2021 |

## 機能

- **従業員レコード** — 部署、ステータス、入社日付きのコア従業員データ
- **勤怠管理** — 出退勤打刻と労働時間計算（分単位）
- **給与計算** — 基本給、残業倍率、控除、手取り額の算出
- **休暇管理** — 有給残日数追跡と申請/承認ワークフロー
- **シフトスケジューリング** — 従業員・日付ごとの開始/終了時刻によるシフト定義と割当
- **人事評価** — 評価期間と評価者追跡付きスコアリングシステム

## アーキテクチャ

```
alice-hrm (lib.rs — 単一ファイルクレート)
├── Date / Time                  # 日付・時刻プリミティブ（外部依存なし）
├── Employee / EmploymentStatus  # 従業員レコード
├── AttendanceRecord             # 出退勤追跡
├── PayrollEntry                 # 給与計算
├── LeaveRequest / LeaveBalance  # 休暇管理
├── Shift / ShiftSchedule        # シフト計画
├── Evaluation                   # 人事評価
└── HrmEngine                    # トップレベルHRオーケストレーター
```

## クイックスタート

```rust
use alice_hrm::{HrmEngine, Date, Time};

let mut hrm = HrmEngine::new();
let emp_id = hrm.add_employee("Alice", 1, Date::new(2025, 1, 15), 300_000);
hrm.clock_in(emp_id, Date::new(2025, 3, 10), Time::new(9, 0));
hrm.clock_out(emp_id, Date::new(2025, 3, 10), Time::new(18, 0));
```

## ビルド

```bash
cargo build
cargo test
cargo clippy -- -W clippy::all
```

## ライセンス

`AGPL-3.0 OR LicenseRef-Commercial` — デュアルライセンス どちらかを選べる

| 選択肢 | 条文 | こういう時 |
|--------|------|-----------|
| **AGPL-3.0** | [LICENSE-AGPL](LICENSE-AGPL) — 無償、報告義務なし | 自分の project も AGPL 互換の OSS、または社内利用のみ |
| **商用ライセンス** | [LICENSE-COMMERCIAL.md](LICENSE-COMMERCIAL.md) — 有償、コピーレフト義務を解除 | クローズドソース製品 / 商用 SaaS / エッジ・ファームウェア配布 / plugin 再配布 / ソース開示を禁じるプラットフォーム NDA |

AGPL は強いコピーレフト: `alice-hrm` を link して配布 / 提供する製品・ファームウェア・
サービスは AGPL で公開する義務がある これはオープンなエコシステムのための意図的な
選択で、それが実行できない場合のために商用ライセンスを用意している

商用ライセンスの問い合わせ: <contact@extoria.co.jp>
