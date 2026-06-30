# Golden Image and Tolerance Policy

Status: Task 10.2 policy baseline
Updated: 2026-06-11

## Purpose

This policy defines what SilicaRAW may compare automatically, what needs manual visual review, and which RAW/color claims remain forbidden until fixture-backed evidence exists.

Task 10.2 does not add golden images, RAW decoding, Core Image probing, ICC parsing, pixel tolerance logic, real fixture files, or color correctness proof.

## Current Evidence Boundary

The repository currently has:

```txt
- a fixture manifest contract for legal RAW/color fixture expectations
- synthetic local-alpha supported-raster smoke fixtures
- original-file hash safety checks
- JPEG sRGB export smoke coverage
- fixture-backed profile probe evidence for local ignored Color Class F fixtures
- JPEG export ICC embedding proof for default sRGB and explicit Display P3 API paths
```

The repository does not yet have:

```txt
- legal committed RAW fixture corpus
- fixture-backed Core Image RAW probe results
- approved golden image outputs
- approved pixel tolerance thresholds
- executed manual Preview.app or Photos review records
```

## Comparison Classes

### Byte Equality

Byte equality is allowed for identity and safety checks:

```txt
original file hash before workflow == original file hash after workflow
fixture manifest expected hash == fixture source hash when a real fixture corpus exists
export output path != original source path
```

Byte equality must not be used to claim color correctness for rendered previews or exported images unless a future task explicitly defines a deterministic render target.

### File and Profile Inspection

File and profile inspection may prove that a file has expected structure or declared metadata:

```txt
JPEG file starts and ends with valid marker bytes
export embeds an ICC profile when the export contract requires one
fixture metadata declares sRGB, Display P3, or untagged policy
```

File and profile inspection does not prove visual color correctness by itself.

### Pixel or Perceptual Tolerance

Pixel or perceptual tolerance is blocked until a later fixture-backed task defines:

```txt
input fixture IDs
decoder and renderer version
host OS version
working space
display/output profile
reference output source
comparison color space
allowed absolute error
allowed relative error
allowed aggregate error
channels and alpha handling
metadata fields excluded from comparison
```

Until those fields are documented, no automated pixel comparison may be used as evidence for color correctness.

### Manual Visual Review

Manual visual review is required for early color trust claims even after automated checks exist.

Manual review must record:

```txt
reviewer
date
git commit
fixture manifest ID
app artifact or command
macOS version
display model or display profile
viewer used: Preview.app or Photos
observed issue notes
pass/fail result
```

Manual review is not a replacement for fixture hashes or profile checks. It is an additional gate for user-facing color confidence. SilicaRAW may be the app under test, but the manual review viewer for this gate must be Preview.app or Photos.

## Claim Rules

Allowed now:

```txt
The fixture manifest records RAW/color expectations.
The local alpha preserves original files in the covered workflow.
JPEG sRGB export smoke coverage exists for synthetic local-alpha fixtures.
JPEG export ICC embedding proof exists for default sRGB and explicit Display P3 API paths.
RAW and color correctness proof remains pending.
```

Forbidden now:

```txt
SilicaRAW supports RAW decoding.
SilicaRAW has proven Fuji RAF support.
SilicaRAW has proven Apple ProRAW support.
SilicaRAW is color correct.
SilicaRAW matches Preview.app, Photos, Lightroom, or any camera vendor renderer.
SilicaRAW preserves Display P3 appearance across displays.
SilicaRAW has a validated golden image baseline.
```

## Future Graduation Gates

RAW support claims require:

```txt
legal fixture manifest entry
fixture source hash verification
Core Image probe result
decode result record
original file hash preservation proof
known unsupported classes recorded as blocked
```

Color correctness claims require:

```txt
legal Color Class F fixture entry
profile inspection record
reference output or reference viewer record
approved tolerance values
automated comparison result
manual visual review record
export ICC embedding proof
```

Export profile claims require:

```txt
export command or app workflow evidence
output file hash
embedded ICC/profile inspection
separate output path proof
original file hash preservation proof
```

## LLM Agent Rules

Do not infer color correctness from:

```txt
fixture manifest entries
compile success
JPEG export success
profile labels
manual screenshots
Spike 003 direction
```

Do not infer RAW support from:

```txt
file extension recognition
placeholder RAW entries
Core Image being selected as the preferred path
fixture metadata without probe output
```

If evidence is missing, record the missing gate instead of adding a fallback, fake fixture, or claim.
