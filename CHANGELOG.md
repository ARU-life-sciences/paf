# Changelog

## 0.3.0 - 2026-11-25

This release fixes several bugs that lost or misread data, and reworks the
record and tag types. It is a breaking release; see "Upgrading" below.

### Fixed

- Tag values containing `:` were cut short. `cs:Z::303*cg:51` was read as an
  empty string; it is now read whole. The same applied to `ds` and any `Z` value.
- Tags other than the 17 built-in ones (for example `ds`, or tags copied in by
  minimap2 `-y`) made the whole record fail to parse. Any tag is now accepted,
  and `H` and `B` values keep their type when written back.
- Typed accessors such as `record.tp()` panicked when a tag had an unexpected
  type (for example `tp:i:5`). They now return `None`.
- Error line numbers were off by one, and were not updated by `read_record`.
  Errors now carry the correct 1-based line and the column or tag at fault:
  `line 2, field query_len: invalid integer: invalid digit found in string`.
- Blank lines, including a trailing blank line, caused an error. They are now
  skipped.
- Tags were written in a random order. They now keep their original order.
- Floats were written to 4 decimal places (`0.00012345` became `0.0001`).
  They are now written in full.
- `Writer::from_path` wrote straight to the file without a buffer, making it
  about 130 times slower. It now uses a `BufWriter`.
- A persistent I/O error made the record iterators yield errors forever. They
  now stop after an I/O error. Invalid UTF-8 is reported as an error on that
  line, and reading continues.
- The writer wrote any record, even ones that produced a corrupt file. It
  now rejects records whose names are empty or whose names or tag values
  contain a tab or line break.

### Added

- `Strand` enum, `Tag::new`, `"NM:i:5".parse::<Tag>()`, and `Display` for
  `PafRecord`, `Tag`, `Type` and `Strand` (a record displays as its PAF line).
- `PafRecord::tag`, `tag_mut`, `set_tag` and `remove_tag`, and a `ds()` accessor.
- `Type::Hex` and `Type::Array` for SAM `H` and `B` values, and `Type::type_code`.
- `Writer::flush`, `get_ref`, `get_mut` and `into_inner`.
- `Reader::line`, giving the number of lines read so far.
- `Error::line`, `Error::field` and `Error::source`.
- `Clone` and `PartialEq` for records, tags and values.

### Changed

- Coordinates and lengths are `u64` instead of `u32`, so sequences over 4.29 Gbp work.
- Floats are written in their shortest exact form, so minimap2's zero-padded
  values change text but not value: `de:f:0.0110` is written as `de:f:0.011`.
- The reader is stricter in three places. It rejects tag types other than
  SAM's `A`, `i`, `f`, `Z`, `H` and `B` (these used to be read as strings),
  `A` values that are not exactly one character (it used to keep the first),
  and empty sequence names.
- The sample data and integration tests are no longer included in the published crate.

### Upgrading from 0.2

- **Building records:** `PafRecord::new(...)` is gone. The fields are now
  public; use a struct literal (see the README), or `..Default::default()`.
- **Reading fields:** use the fields directly (`record.query_name`). The old
  getter methods still work but are deprecated.
- **Strand:** `record.strand` is a `Strand` (`Strand::Forward`/`Strand::Reverse`),
  not a `char`. Use `record.strand.as_char()` for `'+'`/`'-'`.
- **Tags:** `optional_fields()` returning a `HashMap<String, Tag>` is replaced
  by the `tags: Vec<Tag>` field. Instead of `Tag::tp(Type::Char('P'))`, write
  `Tag::new("tp", Type::Char('P'))?` or `"tp:A:P".parse()?`.
- **Tag accessors** return values rather than references: `record.nm()` is
  `Option<i64>` (was `Option<&i64>`), and `record.cg()` is `Option<&str>` (was
  `Option<&String>`). On `Type`, `get_int`/`get_float`/`get_string`/`get_char`
  are now `as_int`/`as_float`/`as_str`/`as_char`.
- **`Writer::from_path`** returns `Writer<BufWriter<File>>`. Call
  `writer.flush()?` when finished so that write errors are reported.
- **`ErrorKind`** has new variants (`InvalidTag`, `WriteRecord`) and is
  `#[non_exhaustive]`, so a `match` on it needs a `_` arm.
