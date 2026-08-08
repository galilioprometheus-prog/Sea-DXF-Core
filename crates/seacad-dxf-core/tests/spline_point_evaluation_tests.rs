use std::{error::Error, io, mem::size_of};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DXF_SPLINE_EVALUATION_MAX_DEGREE, DxfAcadVersion, DxfAsciiRawDocument,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfDouble, DxfError,
    DxfMemorySource, DxfReadOptions, DxfResourceProfile, DxfSplineAnalyticDirectory,
    DxfSplineEvaluatedPoint, DxfSplineEvaluationInputKind, DxfSplinePointEvaluation,
    DxfSplinePointEvaluationIssue, DxfSplinePointEvaluationState, NoopDxfReadObserver,
};

#[test]
fn every_dialect_evaluates_quadratic_endpoints_and_midpoint() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = quadratic_ascii(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.spline_analytic_directory(&DxfCancellationToken::default())?;

        let binary_bytes = quadratic_binary(version, false)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary.spline_analytic_directory(&DxfCancellationToken::default())?;

        let expected = [[0.0, 0.0, 0.0], [1.25, 1.0, 0.0], [3.0, 0.0, 0.0]];
        assert_samples(&ascii_directory, expected)?;
        assert_samples(&binary_directory, expected)?;
        assert_eq!(
            sample_bits(&ascii_directory)?,
            sample_bits(&binary_directory)?
        );
    }
    Ok(())
}

#[test]
fn rational_weights_are_evaluated_in_homogeneous_coordinates() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nSPLINE\n70\n4\n71\n1\n72\n4\n73\n2\n\
40\n0\n40\n0\n40\n1\n40\n1\n\
10\n0\n20\n0\n41\n1\n10\n2\n20\n0\n41\n3\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.spline_analytic_directory(&DxfCancellationToken::default())?;
    let raw = first_raw(&directory)?;
    let evaluation = available(directory.evaluate_point_for_raw_record(
        raw,
        DxfDouble::from_f64(0.5),
        &DxfCancellationToken::default(),
    )?)?;

    assert_eq!(
        evaluation.components().map(DxfDouble::to_f64),
        [1.5, 0.0, 0.0]
    );
    Ok(())
}

#[test]
fn unavailable_domain_and_degree_limit_states_are_typed() -> Result<(), Box<dyn Error>> {
    let bytes = quadratic_ascii("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.spline_analytic_directory(&DxfCancellationToken::default())?;
    let raw = first_raw(&directory)?;

    assert!(matches!(
        state(directory.evaluate_point_for_raw_record(
            raw,
            DxfDouble::from_f64(f64::NAN),
            &DxfCancellationToken::default(),
        )?)?,
        DxfSplinePointEvaluationState::Unavailable(
            DxfSplinePointEvaluationIssue::NonFiniteParameter
        )
    ));
    assert!(matches!(
        state(directory.evaluate_point_for_raw_record(
            raw,
            DxfDouble::from_f64(-0.25),
            &DxfCancellationToken::default(),
        )?)?,
        DxfSplinePointEvaluationState::Unavailable(
            DxfSplinePointEvaluationIssue::ParameterOutOfDomain { .. }
        )
    ));

    let unavailable_bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nSPLINE\n10\n0\n20\n0\n0\nENDSEC\n0\nEOF\n";
    let unavailable_source = DxfMemorySource::new(unavailable_bytes, DxfResourceProfile::Safe)?;
    let unavailable_document = open_ascii(&unavailable_source)?;
    let unavailable_directory =
        unavailable_document.spline_analytic_directory(&DxfCancellationToken::default())?;
    let unavailable_raw = first_raw(&unavailable_directory)?;
    assert!(matches!(
        state(unavailable_directory.evaluate_point_for_raw_record(
            unavailable_raw,
            DxfDouble::from_f64(0.0),
            &DxfCancellationToken::default(),
        )?)?,
        DxfSplinePointEvaluationState::Unavailable(
            DxfSplinePointEvaluationIssue::AnalyticUnavailable(_)
        )
    ));

    let degree_bytes = high_degree_ascii();
    let degree_source = DxfMemorySource::new(&degree_bytes, DxfResourceProfile::Safe)?;
    let degree_document = open_ascii(&degree_source)?;
    let degree_directory =
        degree_document.spline_analytic_directory(&DxfCancellationToken::default())?;
    let degree_raw = first_raw(&degree_directory)?;
    assert_eq!(DXF_SPLINE_EVALUATION_MAX_DEGREE, 64);
    assert_eq!(
        state(degree_directory.evaluate_point_for_raw_record(
            degree_raw,
            DxfDouble::from_f64(0.5),
            &DxfCancellationToken::default(),
        )?)?,
        DxfSplinePointEvaluationState::Unavailable(
            DxfSplinePointEvaluationIssue::DegreeLimitExceeded {
                degree: 65,
                maximum: 64,
            }
        )
    );
    Ok(())
}

#[test]
fn nonfinite_input_cancellation_lookup_and_traits_are_explicit() -> Result<(), Box<dyn Error>> {
    let bytes = quadratic_binary(DxfAcadVersion::Ac1032, true)?;
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_binary(&source)?;
    let directory = document.spline_analytic_directory(&DxfCancellationToken::default())?;
    let raw = first_raw(&directory)?;
    assert!(matches!(
        state(directory.evaluate_point_for_raw_record(
            raw,
            DxfDouble::from_f64(0.5),
            &DxfCancellationToken::default(),
        )?)?,
        DxfSplinePointEvaluationState::Unavailable(DxfSplinePointEvaluationIssue::NonFiniteInput {
            kind: DxfSplineEvaluationInputKind::ControlPoint,
            ..
        })
    ));

    let cancelled = DxfCancellationToken::default();
    cancelled.cancel();
    assert!(matches!(
        directory.evaluate_point_for_raw_record(raw, DxfDouble::from_f64(0.5), &cancelled),
        Err(DxfError::Cancelled)
    ));
    assert_eq!(
        directory.evaluate_point_for_raw_record(
            u64::MAX,
            DxfDouble::from_f64(0.5),
            &DxfCancellationToken::default(),
        )?,
        None
    );
    assert_copy::<DxfSplineEvaluatedPoint>();
    assert_copy::<DxfSplinePointEvaluation>();
    assert_send_sync::<DxfSplinePointEvaluation>();
    assert!(size_of::<DxfSplinePointEvaluation>() <= 128);
    Ok(())
}

fn assert_samples(
    directory: &DxfSplineAnalyticDirectory,
    expected: [[f64; 3]; 3],
) -> Result<(), Box<dyn Error>> {
    let raw = first_raw(directory)?;
    for (parameter, expected) in [0.0, 0.5, 1.0].into_iter().zip(expected) {
        let point = available(directory.evaluate_point_for_raw_record(
            raw,
            DxfDouble::from_f64(parameter),
            &DxfCancellationToken::default(),
        )?)?;
        assert_eq!(point.components().map(DxfDouble::to_f64), expected);
    }
    Ok(())
}

fn sample_bits(directory: &DxfSplineAnalyticDirectory) -> Result<Vec<[u64; 3]>, Box<dyn Error>> {
    let raw = first_raw(directory)?;
    [0.0, 0.5, 1.0]
        .into_iter()
        .map(|parameter| {
            let point = available(directory.evaluate_point_for_raw_record(
                raw,
                DxfDouble::from_f64(parameter),
                &DxfCancellationToken::default(),
            )?)?;
            Ok(point.components().map(DxfDouble::to_bits))
        })
        .collect()
}

fn first_raw(directory: &DxfSplineAnalyticDirectory) -> Result<u64, io::Error> {
    directory
        .entries()
        .first()
        .map(|entry| entry.record.record().ordinal())
        .ok_or_else(|| io::Error::other("missing spline"))
}

fn state(
    evaluation: Option<DxfSplinePointEvaluation>,
) -> Result<DxfSplinePointEvaluationState, io::Error> {
    evaluation
        .map(DxfSplinePointEvaluation::state)
        .ok_or_else(|| io::Error::other("missing evaluation"))
}

fn available(
    evaluation: Option<DxfSplinePointEvaluation>,
) -> Result<DxfSplineEvaluatedPoint, io::Error> {
    match state(evaluation)? {
        DxfSplinePointEvaluationState::Available(point) => Ok(point),
        DxfSplinePointEvaluationState::Unavailable(issue) => Err(io::Error::other(format!(
            "unavailable evaluation: {issue:?}"
        ))),
        _ => Err(io::Error::other("unknown evaluation state")),
    }
}

fn quadratic_ascii(version: &str) -> Vec<u8> {
    format!(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\n{version}\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nSPLINE\n70\n0\n71\n2\n72\n6\n73\n3\n\
40\n0\n40\n0\n40\n0\n40\n1\n40\n1\n40\n1\n\
10\n0\n20\n0\n10\n1\n20\n2\n10\n3\n20\n0\n0\nENDSEC\n0\nEOF\n"
    )
    .into_bytes()
}

fn high_degree_ascii() -> Vec<u8> {
    let mut text = String::from(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nSPLINE\n70\n0\n71\n65\n72\n132\n73\n66\n",
    );
    for _ in 0..66 {
        text.push_str("40\n0\n");
    }
    for _ in 0..66 {
        text.push_str("40\n1\n");
    }
    for ordinal in 0..66 {
        text.push_str(&format!("10\n{ordinal}\n20\n0\n"));
    }
    text.push_str("0\nENDSEC\n0\nEOF\n");
    text.into_bytes()
}

fn quadratic_binary(version: DxfAcadVersion, nonfinite: bool) -> io::Result<Vec<u8>> {
    let mut bytes = DXF_BINARY_SENTINEL.to_vec();
    for (code, value) in [
        (0, b"SECTION".as_slice()),
        (2, b"HEADER"),
        (9, b"$ACADVER"),
        (1, version.code().as_bytes()),
        (0, b"ENDSEC"),
        (0, b"SECTION"),
        (2, b"ENTITIES"),
        (0, b"SPLINE"),
    ] {
        push_string(&mut bytes, version, code, value)?;
    }
    for (code, value) in [(70, 0), (71, 2), (72, 6), (73, 3)] {
        push_i16(&mut bytes, version, code, value)?;
    }
    for knot in [0.0, 0.0, 0.0, 1.0, 1.0, 1.0] {
        push_double(&mut bytes, version, 40, knot)?;
    }
    for (ordinal, (x, y)) in [(0.0, 0.0), (1.0, 2.0), (3.0, 0.0)].into_iter().enumerate() {
        let x = if nonfinite && ordinal == 1 {
            f64::INFINITY
        } else {
            x
        };
        push_double(&mut bytes, version, 10, x)?;
        push_double(&mut bytes, version, 20, y)?;
    }
    push_string(&mut bytes, version, 0, b"ENDSEC")?;
    push_string(&mut bytes, version, 0, b"EOF")?;
    Ok(bytes)
}

fn push_string(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    code: i16,
    value: &[u8],
) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(value);
    bytes.push(0);
    Ok(())
}

fn push_i16(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16, value: i16) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(&value.to_le_bytes());
    Ok(())
}

fn push_double(
    bytes: &mut Vec<u8>,
    version: DxfAcadVersion,
    code: i16,
    value: f64,
) -> io::Result<()> {
    push_code(bytes, version, code)?;
    bytes.extend_from_slice(&value.to_bits().to_le_bytes());
    Ok(())
}

fn push_code(bytes: &mut Vec<u8>, version: DxfAcadVersion, code: i16) -> io::Result<()> {
    if version == DxfAcadVersion::Ac1009 {
        bytes.push(u8::try_from(code).map_err(|_| io::Error::other("group code"))?);
    } else {
        bytes.extend_from_slice(&code.to_le_bytes());
    }
    Ok(())
}

fn open_ascii<'a>(source: &'a dyn DxfByteSource) -> Result<DxfAsciiRawDocument<'a>, DxfError> {
    let mut observer = NoopDxfReadObserver;
    DxfAsciiRawDocument::open(
        source,
        DxfReadOptions::strict(),
        &DxfCancellationToken::default(),
        &mut observer,
    )
}

fn open_binary<'a>(source: &'a dyn DxfByteSource) -> Result<DxfBinaryRawDocument<'a>, DxfError> {
    let mut observer = NoopDxfReadObserver;
    DxfBinaryRawDocument::open(
        source,
        DxfReadOptions::strict(),
        &DxfCancellationToken::default(),
        &mut observer,
    )
}

fn assert_copy<T: Copy>() {}
fn assert_send_sync<T: Send + Sync>() {}
