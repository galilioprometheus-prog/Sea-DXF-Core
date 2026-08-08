use std::{error::Error, io, mem::size_of};

use seacad_dxf_core::{
    DXF_BINARY_SENTINEL, DxfAcadVersion, DxfAsciiRawDocument, DxfBinaryRawDocument, DxfByteSource,
    DxfCancellationToken, DxfDouble, DxfError, DxfMemorySource, DxfReadOptions, DxfResourceProfile,
    DxfSplineAnalyticDirectory, DxfSplineEvaluatedDifferential, DxfSplineEvaluatedVector,
    DxfSplineFirstDerivativeEvaluation, DxfSplineFirstDerivativeState,
    DxfSplinePointEvaluationIssue, NoopDxfReadObserver,
};

#[test]
fn every_dialect_has_quadratic_first_derivative_parity() -> Result<(), Box<dyn Error>> {
    for version in DxfAcadVersion::SUPPORTED {
        let ascii_bytes = quadratic_ascii(version.code());
        let ascii_source = DxfMemorySource::new(&ascii_bytes, DxfResourceProfile::Safe)?;
        let ascii = open_ascii(&ascii_source)?;
        let ascii_directory = ascii.spline_analytic_directory(&DxfCancellationToken::default())?;

        let binary_bytes = quadratic_binary(version)?;
        let binary_source = DxfMemorySource::new(&binary_bytes, DxfResourceProfile::Safe)?;
        let binary = open_binary(&binary_source)?;
        let binary_directory = binary.spline_analytic_directory(&DxfCancellationToken::default())?;

        let expected_points = [[0.0, 0.0, 0.0], [1.25, 1.0, 0.0], [3.0, 0.0, 0.0]];
        let expected_derivatives = [[2.0, 4.0, 0.0], [3.0, 0.0, 0.0], [4.0, -4.0, 0.0]];
        assert_samples(&ascii_directory, expected_points, expected_derivatives)?;
        assert_samples(&binary_directory, expected_points, expected_derivatives)?;
        assert_eq!(
            sample_bits(&ascii_directory)?,
            sample_bits(&binary_directory)?
        );
    }
    Ok(())
}

#[test]
fn rational_first_derivative_uses_the_quotient_rule() -> Result<(), Box<dyn Error>> {
    let bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nSPLINE\n70\n4\n71\n1\n72\n4\n73\n2\n\
40\n0\n40\n0\n40\n1\n40\n1\n\
10\n0\n20\n0\n41\n1\n10\n2\n20\n0\n41\n3\n0\nENDSEC\n0\nEOF\n";
    let source = DxfMemorySource::new(bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.spline_analytic_directory(&DxfCancellationToken::default())?;
    let differential = available(evaluate(&directory, 0.5)?)?;

    assert_eq!(
        differential.point().components().map(DxfDouble::to_f64),
        [1.5, 0.0, 0.0]
    );
    assert_eq!(
        differential
            .first_derivative()
            .components()
            .map(DxfDouble::to_f64),
        [1.5, 0.0, 0.0]
    );
    Ok(())
}

#[test]
fn parameter_and_degree_failures_reuse_the_point_contract() -> Result<(), Box<dyn Error>> {
    let bytes = quadratic_ascii("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.spline_analytic_directory(&DxfCancellationToken::default())?;
    let raw = first_raw(&directory)?;

    assert!(matches!(
        state(directory.evaluate_first_derivative_for_raw_record(
            raw,
            DxfDouble::from_f64(f64::INFINITY),
            &DxfCancellationToken::default(),
        )?)?,
        DxfSplineFirstDerivativeState::Unavailable(
            DxfSplinePointEvaluationIssue::NonFiniteParameter
        )
    ));
    assert!(matches!(
        state(directory.evaluate_first_derivative_for_raw_record(
            raw,
            DxfDouble::from_f64(2.0),
            &DxfCancellationToken::default(),
        )?)?,
        DxfSplineFirstDerivativeState::Unavailable(
            DxfSplinePointEvaluationIssue::ParameterOutOfDomain { .. }
        )
    ));

    let degree_bytes = high_degree_ascii();
    let degree_source = DxfMemorySource::new(&degree_bytes, DxfResourceProfile::Safe)?;
    let degree_document = open_ascii(&degree_source)?;
    let degree_directory =
        degree_document.spline_analytic_directory(&DxfCancellationToken::default())?;
    let degree_raw = first_raw(&degree_directory)?;
    assert!(matches!(
        state(degree_directory.evaluate_first_derivative_for_raw_record(
            degree_raw,
            DxfDouble::from_f64(0.5),
            &DxfCancellationToken::default(),
        )?)?,
        DxfSplineFirstDerivativeState::Unavailable(
            DxfSplinePointEvaluationIssue::DegreeLimitExceeded {
                degree: 65,
                maximum: 64
            }
        )
    ));
    Ok(())
}

#[test]
fn unavailable_cancellation_lookup_and_traits_are_explicit() -> Result<(), Box<dyn Error>> {
    let unavailable_bytes = b"0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1032\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nSPLINE\n10\n0\n20\n0\n0\nENDSEC\n0\nEOF\n";
    let unavailable_source = DxfMemorySource::new(unavailable_bytes, DxfResourceProfile::Safe)?;
    let unavailable_document = open_ascii(&unavailable_source)?;
    let unavailable_directory =
        unavailable_document.spline_analytic_directory(&DxfCancellationToken::default())?;
    let unavailable_raw = first_raw(&unavailable_directory)?;
    assert!(matches!(
        state(
            unavailable_directory.evaluate_first_derivative_for_raw_record(
                unavailable_raw,
                DxfDouble::from_f64(0.0),
                &DxfCancellationToken::default(),
            )?
        )?,
        DxfSplineFirstDerivativeState::Unavailable(
            DxfSplinePointEvaluationIssue::AnalyticUnavailable(_)
        )
    ));

    let bytes = quadratic_ascii("AC1032");
    let source = DxfMemorySource::new(&bytes, DxfResourceProfile::Safe)?;
    let document = open_ascii(&source)?;
    let directory = document.spline_analytic_directory(&DxfCancellationToken::default())?;
    let raw = first_raw(&directory)?;
    let cancelled = DxfCancellationToken::default();
    cancelled.cancel();
    assert!(matches!(
        directory.evaluate_first_derivative_for_raw_record(
            raw,
            DxfDouble::from_f64(0.5),
            &cancelled
        ),
        Err(DxfError::Cancelled)
    ));
    assert_eq!(
        directory.evaluate_first_derivative_for_raw_record(
            u64::MAX,
            DxfDouble::from_f64(0.5),
            &DxfCancellationToken::default(),
        )?,
        None
    );
    assert_copy::<DxfSplineEvaluatedVector>();
    assert_copy::<DxfSplineEvaluatedDifferential>();
    assert_copy::<DxfSplineFirstDerivativeEvaluation>();
    assert_send_sync::<DxfSplineFirstDerivativeEvaluation>();
    assert!(size_of::<DxfSplineFirstDerivativeEvaluation>() <= 192);
    Ok(())
}

fn assert_samples(
    directory: &DxfSplineAnalyticDirectory,
    expected_points: [[f64; 3]; 3],
    expected_derivatives: [[f64; 3]; 3],
) -> Result<(), Box<dyn Error>> {
    for ((parameter, point), derivative) in [0.0, 0.5, 1.0]
        .into_iter()
        .zip(expected_points)
        .zip(expected_derivatives)
    {
        let differential = available(evaluate(directory, parameter)?)?;
        assert_eq!(
            differential.point().components().map(DxfDouble::to_f64),
            point
        );
        assert_eq!(
            differential
                .first_derivative()
                .components()
                .map(DxfDouble::to_f64),
            derivative
        );
    }
    Ok(())
}

type DifferentialBits = ([u64; 3], [u64; 3]);

fn sample_bits(
    directory: &DxfSplineAnalyticDirectory,
) -> Result<Vec<DifferentialBits>, Box<dyn Error>> {
    [0.0, 0.5, 1.0]
        .into_iter()
        .map(|parameter| {
            let differential = available(evaluate(directory, parameter)?)?;
            Ok((
                differential.point().components().map(DxfDouble::to_bits),
                differential
                    .first_derivative()
                    .components()
                    .map(DxfDouble::to_bits),
            ))
        })
        .collect()
}

fn evaluate(
    directory: &DxfSplineAnalyticDirectory,
    parameter: f64,
) -> Result<Option<DxfSplineFirstDerivativeEvaluation>, Box<dyn Error>> {
    let raw = first_raw(directory)?;
    Ok(directory.evaluate_first_derivative_for_raw_record(
        raw,
        DxfDouble::from_f64(parameter),
        &DxfCancellationToken::default(),
    )?)
}

fn first_raw(directory: &DxfSplineAnalyticDirectory) -> Result<u64, io::Error> {
    directory
        .entries()
        .first()
        .map(|entry| entry.record.record().ordinal())
        .ok_or_else(|| io::Error::other("missing spline"))
}

fn state(
    evaluation: Option<DxfSplineFirstDerivativeEvaluation>,
) -> Result<DxfSplineFirstDerivativeState, io::Error> {
    evaluation
        .map(DxfSplineFirstDerivativeEvaluation::state)
        .ok_or_else(|| io::Error::other("missing derivative"))
}

fn available(
    evaluation: Option<DxfSplineFirstDerivativeEvaluation>,
) -> Result<DxfSplineEvaluatedDifferential, io::Error> {
    match state(evaluation)? {
        DxfSplineFirstDerivativeState::Available(value) => Ok(value),
        DxfSplineFirstDerivativeState::Unavailable(issue) => Err(io::Error::other(format!(
            "unavailable derivative: {issue:?}"
        ))),
        _ => Err(io::Error::other("unknown derivative state")),
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

fn quadratic_binary(version: DxfAcadVersion) -> io::Result<Vec<u8>> {
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
    for (x, y) in [(0.0, 0.0), (1.0, 2.0), (3.0, 0.0)] {
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
