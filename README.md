# Coherence demo: 1000 € на троих + 1 цент

Пример к посту: «делим 1000 € на троих, получается по 333,33, и плюс один цент ещё кому-то достанется».

Здесь задача поставлена не `markdown.md`, а графом спеков и acceptance criteria (AC),
у которых есть исполняемое доказательство. Каталог лежит в Dolt, экспорт — в
`.coherence/exports/catalog.jsonl`.

## Почему не markdown

Из markdown можно вытащить семантику, но нельзя получить ответ «выполнено / не
выполнено». Здесь обещание — узел графа, критерий — узел, доказательство —
команда. Проверка не требует загружать AST в голову: запустил `verify-spec` и
получил PASS/FAIL по каждому утверждению.

Под капотом — обычные таблицы: `specs`, `acceptance_criteria`,
`spec_relations`, `codeintel_code_locations`, `codeintel_ac_links`. Это не
магический формат: те же узлы и типизированные рёбра можно держать в SQL,
Neo4j или чём угодно. Важна не база, а то, что смысл записан узлами и связями,
а не абзацами.

## Что именно обещает система

Три спеки, связанные в граф:

```text
SPEC-bill-split  (product)
├── has → AC-split-1000-across-3
├── has → AC-split-total-preserved
├── has → AC-split-equal-division-has-no-remainder
├── has → AC-split-rejects-zero-participants
├── depends_on → SPEC-remainder-assignment
└── constrained_by → SPEC-eur-minor-units

SPEC-remainder-assignment  (component)
├── has → AC-remainder-seeded-choice-reproducible
├── has → AC-remainder-one-cent-per-participant
└── depends_on → SPEC-eur-minor-units

SPEC-eur-minor-units  (foundation)
├── has → AC-eur-parses-decimal-to-cents
├── has → AC-eur-rejects-sub-cent-amounts
└── has → AC-eur-format-roundtrip
```

Смысл каждой спеки отдельно:

| Спека | Что означает |
|---|---|
| `SPEC-bill-split` | Разбить сумму в евро на N участников так, чтобы сумма частей **точно** равнялась исходной. Для 1000 € и 3 участников база — 333,33 €, остаток 0,01 € уходит ровно одному участнику. |
| `SPEC-remainder-assignment` | Остаток — не «грязная случайность» внутри кода, а явная политика: seed является входом, порядок получателей детерминирован, один цент не может попасть одному участнику дважды. |
| `SPEC-eur-minor-units` | Деньги — целые центы (`i64`), не `f64`. Парсинг и форматирование — точные десятичные, суммы с суб-центовой точностью отвергаются. |

Ключевая арифметика для 1000 €:

```text
100000 центов / 3 = 33333 цента базы, остаток 1 цент
33333 * 3 = 99999, +1 = 100000
один участник получает 33334 цента = 333,34 €
```

Никакого «примерно 333,33». Либо сумма сходится, либо AC падает.

## Код

Ядро — две функции:

```rust
// src/money.rs
pub fn parse_eur_to_cents(input: &str) -> Result<i64, MoneyError>

// src/split.rs
pub fn split_evenly(
    total_cents: i64,
    participants: usize,
    seed: u64,
) -> Result<SplitResult, SplitError>
```

`split_evenly` считает `base = total / n`, `remainder = total % n`, раздаёт
по одному центу первым `remainder` участникам в детерминированном порядке,
который зависит от `seed`. Поэтому «случайность» воспроизводима: тот же запрос
можно прогнать в тесте и в CI.

```console
$ cargo run --quiet -- 1000.00 3 1
total:        1000.00 EUR
participants: 3
base share:   333.33 EUR
remainder:    0.01 EUR -> [0]
seed:         1

participant 0: 333.34 EUR
participant 1: 333.33 EUR
participant 2: 333.33 EUR

sum:          1000.00 EUR
```

С другим seed цент достаётся другому участнику:

```console
$ cargo run --quiet -- 1000.00 3 42
...
remainder:    0.01 EUR -> [2]
...
participant 2: 333.34 EUR
sum:          1000.00 EUR
```

## Как агент делает задачу

1. Читает не прозу, а граф: `spec list`, `ac list --spec-id ...`, связи
   `depends_on` / `constrained_by`. У каждой AC есть `intent` — проверяемое
   утверждение, а не пожелание.
2. Выбирает границы: целые центы, явный seed, один цент одному получателю.
   Эти решения не «в голове агента», они записаны в спеке и AC.
3. Пишет код в `src/`.
4. Пишет по тесту на каждую AC. `ac-tests materialize-rust` создаёт
   недостающие `tests/ac_<slug>.rs` и записывает в каталог связи
   `AC —verified_by→ test_file` с командой запуска.
5. Запускает `verify-spec`. Coherence берёт из каталога команды и прогоняет
   их. Результат — не «мнение LLM», а `passed/failed` по каждой AC.
6. Если тест упал — либо код неверен, либо спека неверна. Тогда меняется
   код или `intent`/AC, а не «договоримся, что и так красиво».

Именно это снимает проблему из поста: не нужно держать в голове AST и
сверять его с `script.py`. Утверждение и его доказательство лежат в одном
графе, а доказательство можно запустить.

## Как код связан со спекой

В текущем MVP исполняемая связь такая:

```text
SPEC-bill-split
  └── has → AC-split-total-preserved
               └── verified_by → tests/ac_split-total-preserved.rs
                                   └── вызывает src/split.rs::split_evenly
```

`tests/ac_split-total-preserved.rs` — это и есть мост между «что обещали» и
«что написали». Тест импортирует реализацию и проверяет утверждение на
наборе сумм, количеств участников и seed:

```rust
for total in [0, 1, 2, 99, 100, 99_999, 100_000, 100_001, 123_456_789] {
    for participants in 1..=7 {
        let result = split_evenly(total, participants, seed).unwrap();
        let sum: i64 = result.allocations.iter().map(|a| a.cents).sum();
        assert_eq!(sum, total);
    }
}
```

Связь `AC → test` хранится в `codeintel_ac_links` как `verified_by`.
Это не «ссылка для красоты»: `verify-spec` реально запускает команду из
`codeintel_code_locations.test_command` и печатает PASS/FAIL.

`implemented_by` (AC → файл реализации) в схеме есть, но установленный
`coherence-bootstrap 0.1.0` / `coherence-core-db 0.2.0` знает только
`test_file` и `test_command` как виды code location. Поэтому в этом примере
я не кладу в каталог фиктивные `implemented_by`-строки: навигация по
исходникам идёт через шапки модулей (`//! Spec: SPEC-...`) и через тест,
который этот модуль исполняет. Это ограничение MVP-инструмента, а не модели.

## Пруф — это не обязательно тест

AC — это утверждение. Evidence — это команда, которая говорит `0` или не `0`.
Сегодня в примере это Rust-тест:

```text
AC-split-total-preserved
  verified_by → cargo test -p coherence-core-db-bootstrap validates_split_total_preserved
```

Но закон сохранения суммы — хороший кандидат и для формального пруфа.
Ту же AC можно связать с Lean-теоремой:

```text
AC-split-total-preserved
  verified_by → cargo test ... validates_split_total_preserved
  verified_by → lake build BillSplit.conservation
```

Coherence не требует, чтобы evidence был именно Rust-тестом. Ему нужен
исполняемый чекер: тест, property-based тест, Lean/Coq/Isabelle, SMT-проверка,
что угодно, что возвращает код возврата. В текущем CLI автоматизирован только
Rust-материализатор; Lean-команда добавляется как `test_command` code location
(через API/JSONL, пока без отдельной CLI-команды).

## Почему «красивый Клод» здесь не проходит

Типичные решения, которые выглядят рабочими:

- считать во `f64` и округлять каждую долю — сумма может дать 999,99 или
  1000,01, а `AC-split-total-preserved` это ловит;
- округлить независимо и «забить» на цент — не сходится инвариант;
- выбрать случайного получателя через `rand::random()` — `AC-remainder-seeded-choice-reproducible`
  требует воспроизводимости, тест падает;
- отдать цент «первому» без политики — спека требует явный seeded-выбор,
  а не скрытое соглашение.

То есть спека не даёт агенту выбрать «минимум кода в ущерб консистентности»:
у выбора есть falsifiable-критерий, и он запускается одной командой.

## Поднять у себя

Манифест `.coherence/project.toml` не коммитится: он привязан к локальному
пути. Создай свой каталог и импортируй снимок графа:

```bash
git clone git@github.com:konovalov-nk/bill-split-coherence-demo.git
cd bill-split-coherence-demo
coherence-bootstrap project init --slug bill_split_demo
coherence-bootstrap db import-jsonl --env dev \
  --in .coherence/exports/catalog-import.jsonl --confirm
cargo test
coherence-bootstrap verify-spec SPEC-bill-split
```

`catalog-import.jsonl` — без `spec_relations`, потому что импортер 0.2.0
не умеет их вставлять (см. заметки ниже). Полный граф со связями —
в `catalog.jsonl`.

## Воспроизвести локально

```bash
cd /tmp/opencode/example-1000

cargo test
coherence-bootstrap spec list
coherence-bootstrap ac list --spec-id SPEC-bill-split
coherence-bootstrap verify-spec SPEC-bill-split
coherence-bootstrap verify-spec SPEC-remainder-assignment
coherence-bootstrap verify-spec SPEC-eur-minor-units

cargo run --quiet -- 1000.00 3 1
cargo run --quiet -- 1000.00 3 42
```

Локальный каталог — `bill_split_demo_1f59_dev` (см. `.coherence/project.toml`).
Экспорт графа — `.coherence/exports/catalog.jsonl`.

## Файлы

```text
Cargo.toml
src/money.rs                 # SPEC-eur-minor-units
src/split.rs                 # SPEC-bill-split + SPEC-remainder-assignment
src/main.rs                  # демо-вывод
tests/ac_*.rs                # по одному evidence-файлу на AC
.coherence/project.toml      # привязка проекта к каталогу
.coherence/exports/catalog.jsonl
```

## Заметки по MVP

- `db export-jsonl` выгружает `spec_relations` без колонки `note`, а
  `db import-jsonl` вставляет relation без `note`; в схеме `note NOT NULL`.
  Поэтому повторный импорт экспортированных связей падает. В этом примере
  три связи добавлены SQL-ом после импорта; для чистого импорта лежит
  `catalog-import.jsonl` без связей. Сам каталог и `verify-spec` работают.
- `implemented_by` для исходников не поддерживается установленной версией
  (`CodeLocationKind` знает только `test_file`/`test_command`). Это следующий
  маленький шаг для инструмента.
