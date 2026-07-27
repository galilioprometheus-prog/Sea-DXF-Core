# M4.3c3 Autodesk text-control reference and AutoCAD oracle

Status: completed 2026-07-27

No external parser source was inspected, copied, translated, or ported.

## Normative Autodesk references

AutoCAD 2027, **Format Codes for Alternate Text Editor Reference**:

<https://help.autodesk.com/view/ACD/2027/ENU/?guid=GUID-7D8BB40F-5C4E-4AE5-BD75-9ED7112E5967>

The reference defines overline, underline, strikethrough, nonbreaking space,
escaped backslash/braces, color, font, absolute/relative height, stacked text,
tracking, oblique angle, width, alignment, paragraph break, semicolon
termination, and an eight-level brace limit.

AutoCAD 2027, **About Formatting Multiline Text in an Alternate Text Editor**:

<https://help.autodesk.com/view/ACD/2027/ENU/?guid=GUID-6F59DA4A-A790-4316-A79C-2CCE723A30CA>

The examples establish nested blocks, parameter controls, stacked text and
paragraph breaks in real formatted strings.

AutoCAD, **Control Codes and Special Characters Reference**:

<https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-Core/files/GUID-968CBC1D-BA99-4519-ABDD-88419EB2BF92.htm>

This reference defines `%%nnn`, `%%c`, `%%d`, `%%k`, `%%o`, `%%p`, `%%u`,
and `%%%`, and explicitly warns that not every percent control works in
multiline text.

AutoCAD DXF, **MTEXT (DXF)**:

<https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-5E5DB93B-F8D3-4433-ADF7-E92E250D2BAB.htm>

This establishes group 1 and group 3 storage for MTEXT content. Reassembly is
deliberately deferred until record semantics.

## AutoCAD 2027 isolated oracle

Executable:

- version: `26.0.60.0.0`;
- SHA-256:
  `fc6b59c29fea759d556083cf43d9de3657974ef0bf57962d49a19892f729fa77`.

The oracle opened a known-valid AC1009 base drawing, created MTEXT only in
memory, and ran with `/readonly`, `/safemode`, and a unique isolated profile.
`EXPLODE` converted the MTEXT into independently listable TEXT/LINE fragments.
No file produced by AutoCAD is used for verbatim evidence.

| Artifact | SHA-256 |
| --- | --- |
| base DXF | `5f2cab15b518ae14065daed7f16a9f7b6276d0bdb85caaf6cb3cbe7df57fd882` |
| documented-family script | `6fc5974ac07adccb28f7d314964e6f347ac82f40b9c4015edf898489d560cdc8` |
| documented-family UTF-16 stdout | `8a097cfedb25b17ba17e4e6e97e01ed9724f4cc141dd6c4a925e2f3069439280` |
| percent/context script | `2caa2e084cc77606365a52ba4e6069d20800b3ae40353c7effdd548db4f0a057` |
| percent/context UTF-16 stdout | `ab151e205961ff2d8f878f1b74cdf550a200b6c24d2e1d3bfb1c4cc7ed67025f` |

Observed results:

- all documented MTEXT backslash families affected the exploded fragments;
- escaped backslash/braces became literal characters;
- stack input produced two text fragments and a separator line;
- height, color, font, tracking, oblique, and width affected fragment
  properties;
- `\P` produced a new paragraph;
- `%%c`, `%%d`, and `%%p` worked in MTEXT in upper and lower case;
- MTEXT preserved `%%u`, `%%o`, `%%k`, `%%%`, and `%%nnn` as literal content,
  confirming that their interpretation is context-dependent;
- incomplete lowercase `\p` consumed later content in the oracle. Its complete
  grammar is not claimed from this observation, so SeaCad keeps it opaque.

An initial hand-authored direct AC1032 MTEXT fixture (SHA-256
`101b2b7ad7689e444fbb975c9eb652f78958aa3ad76cf4cbc0c69ccd223fc9db`)
caused Core Console to terminate with an access violation before producing a
`LIST` result. Its cause was not inferred, and it is excluded from all support
evidence. The accepted oracle instead starts from the independently known-valid
base drawing above and constructs the entity in memory.

Fixtures, profiles, scripts, logs, and AutoCAD artifacts remain outside the
repository. Only hashes, observations, tests, and this audit are committed.
