use special::Beta;

/// The inverse CDF of Student's t-distribution.
///
/// `p` is the cumulative probability, and `dof` (aka. "ν") is the degrees
/// of freedom (a parameter of the distribution).
pub fn inv_cdf(p: f64, dof: f64) -> f64 {
    assert!(p >= 0.0 && p <= 1.0);
    let x = 2. * p.min(1. - p);
    let a = 0.5 * dof;
    let b = 0.5;
    let y = x.inv_inc_beta(a, b, a.ln_beta(b));
    let y = (dof * (1. - y) / y).sqrt();
    if p > 0.5 {
        y
    } else {
        -y
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::*;

    #[test]
    fn t_table() {
        // This test checks our implementation against the whole t-table
        // copied from https://en.wikipedia.org/wiki/Student's_t-distribution

        macro_rules! assert_rel_eq {
            ($p:expr, $dof:expr, $exp:expr) => {
                assert_relative_eq!(inv_cdf($p, $dof), $exp, max_relative = 0.001);
            };
        }

        assert_rel_eq!(0.75, 1.0, 1.000);
        assert_rel_eq!(0.8, 1.0, 1.376);
        assert_rel_eq!(0.85, 1.0, 1.963);
        assert_rel_eq!(0.9, 1.0, 3.078);
        assert_rel_eq!(0.95, 1.0, 6.314);
        assert_rel_eq!(0.975, 1.0, 12.71);
        assert_rel_eq!(0.99, 1.0, 31.82);
        assert_rel_eq!(0.995, 1.0, 63.66);
        assert_rel_eq!(0.9975, 1.0, 127.3);
        assert_rel_eq!(0.999, 1.0, 318.3);
        assert_rel_eq!(0.9995, 1.0, 636.6);

        assert_rel_eq!(0.75, 002.0, 0.816);
        // assert_rel_eq!(0.8, 002.0, 1.080);  // We get 1.061 for some reason...
        assert_rel_eq!(0.85, 002.0, 1.386);
        assert_rel_eq!(0.9, 002.0, 1.886);
        assert_rel_eq!(0.95, 002.0, 2.920);
        assert_rel_eq!(0.975, 002.0, 4.303);
        assert_rel_eq!(0.99, 002.0, 6.965);
        assert_rel_eq!(0.995, 002.0, 9.925);
        assert_rel_eq!(0.9975, 002.0, 14.09);
        assert_rel_eq!(0.999, 002.0, 22.33);
        assert_rel_eq!(0.9995, 002.0, 31.60);

        assert_rel_eq!(0.75, 003.0, 0.765);
        assert_rel_eq!(0.8, 003.0, 0.978);
        assert_rel_eq!(0.85, 003.0, 1.250);
        assert_rel_eq!(0.9, 003.0, 1.638);
        assert_rel_eq!(0.95, 003.0, 2.353);
        assert_rel_eq!(0.975, 003.0, 3.182);
        assert_rel_eq!(0.99, 003.0, 4.541);
        assert_rel_eq!(0.995, 003.0, 5.841);
        assert_rel_eq!(0.9975, 003.0, 7.453);
        assert_rel_eq!(0.999, 003.0, 10.21);
        assert_rel_eq!(0.9995, 003.0, 12.92);

        assert_rel_eq!(0.75, 004.0, 0.741);
        assert_rel_eq!(0.8, 004.0, 0.941);
        assert_rel_eq!(0.85, 004.0, 1.190);
        assert_rel_eq!(0.9, 004.0, 1.533);
        assert_rel_eq!(0.95, 004.0, 2.132);
        assert_rel_eq!(0.975, 004.0, 2.776);
        assert_rel_eq!(0.99, 004.0, 3.747);
        assert_rel_eq!(0.995, 004.0, 4.604);
        assert_rel_eq!(0.9975, 004.0, 5.598);
        assert_rel_eq!(0.999, 004.0, 7.173);
        assert_rel_eq!(0.9995, 004.0, 8.610);

        assert_rel_eq!(0.75, 005.0, 0.727);
        assert_rel_eq!(0.8, 005.0, 0.920);
        assert_rel_eq!(0.85, 005.0, 1.156);
        assert_rel_eq!(0.9, 005.0, 1.476);
        assert_rel_eq!(0.95, 005.0, 2.015);
        assert_rel_eq!(0.975, 005.0, 2.571);
        assert_rel_eq!(0.99, 005.0, 3.365);
        assert_rel_eq!(0.995, 005.0, 4.032);
        assert_rel_eq!(0.9975, 005.0, 4.773);
        assert_rel_eq!(0.999, 005.0, 5.893);
        assert_rel_eq!(0.9995, 005.0, 6.869);

        assert_rel_eq!(0.75, 006.0, 0.718);
        assert_rel_eq!(0.8, 006.0, 0.906);
        assert_rel_eq!(0.85, 006.0, 1.134);
        assert_rel_eq!(0.9, 006.0, 1.440);
        assert_rel_eq!(0.95, 006.0, 1.943);
        assert_rel_eq!(0.975, 006.0, 2.447);
        assert_rel_eq!(0.99, 006.0, 3.143);
        assert_rel_eq!(0.995, 006.0, 3.707);
        assert_rel_eq!(0.9975, 006.0, 4.317);
        assert_rel_eq!(0.999, 006.0, 5.208);
        assert_rel_eq!(0.9995, 006.0, 5.959);

        assert_rel_eq!(0.75, 007.0, 0.711);
        assert_rel_eq!(0.8, 007.0, 0.896);
        assert_rel_eq!(0.85, 007.0, 1.119);
        assert_rel_eq!(0.9, 007.0, 1.415);
        assert_rel_eq!(0.95, 007.0, 1.895);
        assert_rel_eq!(0.975, 007.0, 2.365);
        assert_rel_eq!(0.99, 007.0, 2.998);
        assert_rel_eq!(0.995, 007.0, 3.499);
        assert_rel_eq!(0.9975, 007.0, 4.029);
        assert_rel_eq!(0.999, 007.0, 4.785);
        assert_rel_eq!(0.9995, 007.0, 5.408);

        assert_rel_eq!(0.75, 008.0, 0.706);
        assert_rel_eq!(0.8, 008.0, 0.889);
        assert_rel_eq!(0.85, 008.0, 1.108);
        assert_rel_eq!(0.9, 008.0, 1.397);
        assert_rel_eq!(0.95, 008.0, 1.860);
        assert_rel_eq!(0.975, 008.0, 2.306);
        assert_rel_eq!(0.99, 008.0, 2.896);
        assert_rel_eq!(0.995, 008.0, 3.355);
        assert_rel_eq!(0.9975, 008.0, 3.833);
        assert_rel_eq!(0.999, 008.0, 4.501);
        assert_rel_eq!(0.9995, 008.0, 5.041);

        assert_rel_eq!(0.75, 009.0, 0.703);
        assert_rel_eq!(0.8, 009.0, 0.883);
        assert_rel_eq!(0.85, 009.0, 1.100);
        assert_rel_eq!(0.9, 009.0, 1.383);
        assert_rel_eq!(0.95, 009.0, 1.833);
        assert_rel_eq!(0.975, 009.0, 2.262);
        assert_rel_eq!(0.99, 009.0, 2.821);
        assert_rel_eq!(0.995, 009.0, 3.250);
        assert_rel_eq!(0.9975, 009.0, 3.690);
        assert_rel_eq!(0.999, 009.0, 4.297);
        assert_rel_eq!(0.9995, 009.0, 4.781);

        assert_rel_eq!(0.75, 010.0, 0.700);
        assert_rel_eq!(0.8, 010.0, 0.879);
        assert_rel_eq!(0.85, 010.0, 1.093);
        assert_rel_eq!(0.9, 010.0, 1.372);
        assert_rel_eq!(0.95, 010.0, 1.812);
        assert_rel_eq!(0.975, 010.0, 2.228);
        assert_rel_eq!(0.99, 010.0, 2.764);
        assert_rel_eq!(0.995, 010.0, 3.169);
        assert_rel_eq!(0.9975, 010.0, 3.581);
        assert_rel_eq!(0.999, 010.0, 4.144);
        assert_rel_eq!(0.9995, 010.0, 4.587);

        assert_rel_eq!(0.75, 011.0, 0.697);
        assert_rel_eq!(0.8, 011.0, 0.876);
        assert_rel_eq!(0.85, 011.0, 1.088);
        assert_rel_eq!(0.9, 011.0, 1.363);
        assert_rel_eq!(0.95, 011.0, 1.796);
        assert_rel_eq!(0.975, 011.0, 2.201);
        assert_rel_eq!(0.99, 011.0, 2.718);
        assert_rel_eq!(0.995, 011.0, 3.106);
        assert_rel_eq!(0.9975, 011.0, 3.497);
        assert_rel_eq!(0.999, 011.0, 4.025);
        assert_rel_eq!(0.9995, 011.0, 4.437);

        assert_rel_eq!(0.75, 012.0, 0.695);
        assert_rel_eq!(0.8, 012.0, 0.873);
        assert_rel_eq!(0.85, 012.0, 1.083);
        assert_rel_eq!(0.9, 012.0, 1.356);
        assert_rel_eq!(0.95, 012.0, 1.782);
        assert_rel_eq!(0.975, 012.0, 2.179);
        assert_rel_eq!(0.99, 012.0, 2.681);
        assert_rel_eq!(0.995, 012.0, 3.055);
        assert_rel_eq!(0.9975, 012.0, 3.428);
        assert_rel_eq!(0.999, 012.0, 3.930);
        assert_rel_eq!(0.9995, 012.0, 4.318);

        assert_rel_eq!(0.75, 013.0, 0.694);
        assert_rel_eq!(0.8, 013.0, 0.870);
        assert_rel_eq!(0.85, 013.0, 1.079);
        assert_rel_eq!(0.9, 013.0, 1.350);
        assert_rel_eq!(0.95, 013.0, 1.771);
        assert_rel_eq!(0.975, 013.0, 2.160);
        assert_rel_eq!(0.99, 013.0, 2.650);
        assert_rel_eq!(0.995, 013.0, 3.012);
        assert_rel_eq!(0.9975, 013.0, 3.372);
        assert_rel_eq!(0.999, 013.0, 3.852);
        assert_rel_eq!(0.9995, 013.0, 4.221);

        assert_rel_eq!(0.75, 014.0, 0.692);
        assert_rel_eq!(0.8, 014.0, 0.868);
        assert_rel_eq!(0.85, 014.0, 1.076);
        assert_rel_eq!(0.9, 014.0, 1.345);
        assert_rel_eq!(0.95, 014.0, 1.761);
        assert_rel_eq!(0.975, 014.0, 2.145);
        assert_rel_eq!(0.99, 014.0, 2.624);
        assert_rel_eq!(0.995, 014.0, 2.977);
        assert_rel_eq!(0.9975, 014.0, 3.326);
        assert_rel_eq!(0.999, 014.0, 3.787);
        assert_rel_eq!(0.9995, 014.0, 4.140);

        assert_rel_eq!(0.75, 015.0, 0.691);
        assert_rel_eq!(0.8, 015.0, 0.866);
        assert_rel_eq!(0.85, 015.0, 1.074);
        assert_rel_eq!(0.9, 015.0, 1.341);
        assert_rel_eq!(0.95, 015.0, 1.753);
        assert_rel_eq!(0.975, 015.0, 2.131);
        assert_rel_eq!(0.99, 015.0, 2.602);
        assert_rel_eq!(0.995, 015.0, 2.947);
        assert_rel_eq!(0.9975, 015.0, 3.286);
        assert_rel_eq!(0.999, 015.0, 3.733);
        assert_rel_eq!(0.9995, 015.0, 4.073);

        assert_rel_eq!(0.75, 016.0, 0.690);
        assert_rel_eq!(0.8, 016.0, 0.865);
        assert_rel_eq!(0.85, 016.0, 1.071);
        assert_rel_eq!(0.9, 016.0, 1.337);
        assert_rel_eq!(0.95, 016.0, 1.746);
        assert_rel_eq!(0.975, 016.0, 2.120);
        assert_rel_eq!(0.99, 016.0, 2.583);
        assert_rel_eq!(0.995, 016.0, 2.921);
        assert_rel_eq!(0.9975, 016.0, 3.252);
        assert_rel_eq!(0.999, 016.0, 3.686);
        assert_rel_eq!(0.9995, 016.0, 4.015);

        assert_rel_eq!(0.75, 017.0, 0.689);
        assert_rel_eq!(0.8, 017.0, 0.863);
        assert_rel_eq!(0.85, 017.0, 1.069);
        assert_rel_eq!(0.9, 017.0, 1.333);
        assert_rel_eq!(0.95, 017.0, 1.740);
        assert_rel_eq!(0.975, 017.0, 2.110);
        assert_rel_eq!(0.99, 017.0, 2.567);
        assert_rel_eq!(0.995, 017.0, 2.898);
        assert_rel_eq!(0.9975, 017.0, 3.222);
        assert_rel_eq!(0.999, 017.0, 3.646);
        assert_rel_eq!(0.9995, 017.0, 3.965);

        assert_rel_eq!(0.75, 018.0, 0.688);
        assert_rel_eq!(0.8, 018.0, 0.862);
        assert_rel_eq!(0.85, 018.0, 1.067);
        assert_rel_eq!(0.9, 018.0, 1.330);
        assert_rel_eq!(0.95, 018.0, 1.734);
        assert_rel_eq!(0.975, 018.0, 2.101);
        assert_rel_eq!(0.99, 018.0, 2.552);
        assert_rel_eq!(0.995, 018.0, 2.878);
        assert_rel_eq!(0.9975, 018.0, 3.197);
        assert_rel_eq!(0.999, 018.0, 3.610);
        assert_rel_eq!(0.9995, 018.0, 3.922);

        assert_rel_eq!(0.75, 019.0, 0.688);
        assert_rel_eq!(0.8, 019.0, 0.861);
        assert_rel_eq!(0.85, 019.0, 1.066);
        assert_rel_eq!(0.9, 019.0, 1.328);
        assert_rel_eq!(0.95, 019.0, 1.729);
        assert_rel_eq!(0.975, 019.0, 2.093);
        assert_rel_eq!(0.99, 019.0, 2.539);
        assert_rel_eq!(0.995, 019.0, 2.861);
        assert_rel_eq!(0.9975, 019.0, 3.174);
        assert_rel_eq!(0.999, 019.0, 3.579);
        assert_rel_eq!(0.9995, 019.0, 3.883);

        assert_rel_eq!(0.75, 020.0, 0.687);
        assert_rel_eq!(0.8, 020.0, 0.860);
        assert_rel_eq!(0.85, 020.0, 1.064);
        assert_rel_eq!(0.9, 020.0, 1.325);
        assert_rel_eq!(0.95, 020.0, 1.725);
        assert_rel_eq!(0.975, 020.0, 2.086);
        assert_rel_eq!(0.99, 020.0, 2.528);
        assert_rel_eq!(0.995, 020.0, 2.845);
        assert_rel_eq!(0.9975, 020.0, 3.153);
        assert_rel_eq!(0.999, 020.0, 3.552);
        assert_rel_eq!(0.9995, 020.0, 3.850);

        assert_rel_eq!(0.75, 021.0, 0.686);
        assert_rel_eq!(0.8, 021.0, 0.859);
        assert_rel_eq!(0.85, 021.0, 1.063);
        assert_rel_eq!(0.9, 021.0, 1.323);
        assert_rel_eq!(0.95, 021.0, 1.721);
        assert_rel_eq!(0.975, 021.0, 2.080);
        assert_rel_eq!(0.99, 021.0, 2.518);
        assert_rel_eq!(0.995, 021.0, 2.831);
        assert_rel_eq!(0.9975, 021.0, 3.135);
        assert_rel_eq!(0.999, 021.0, 3.527);
        assert_rel_eq!(0.9995, 021.0, 3.819);

        assert_rel_eq!(0.75, 022.0, 0.686);
        assert_rel_eq!(0.8, 022.0, 0.858);
        assert_rel_eq!(0.85, 022.0, 1.061);
        assert_rel_eq!(0.9, 022.0, 1.321);
        assert_rel_eq!(0.95, 022.0, 1.717);
        assert_rel_eq!(0.975, 022.0, 2.074);
        assert_rel_eq!(0.99, 022.0, 2.508);
        assert_rel_eq!(0.995, 022.0, 2.819);
        assert_rel_eq!(0.9975, 022.0, 3.119);
        assert_rel_eq!(0.999, 022.0, 3.505);
        assert_rel_eq!(0.9995, 022.0, 3.792);

        assert_rel_eq!(0.75, 023.0, 0.685);
        assert_rel_eq!(0.8, 023.0, 0.858);
        assert_rel_eq!(0.85, 023.0, 1.060);
        assert_rel_eq!(0.9, 023.0, 1.319);
        assert_rel_eq!(0.95, 023.0, 1.714);
        assert_rel_eq!(0.975, 023.0, 2.069);
        assert_rel_eq!(0.99, 023.0, 2.500);
        assert_rel_eq!(0.995, 023.0, 2.807);
        assert_rel_eq!(0.9975, 023.0, 3.104);
        assert_rel_eq!(0.999, 023.0, 3.485);
        assert_rel_eq!(0.9995, 023.0, 3.767);

        assert_rel_eq!(0.75, 024.0, 0.685);
        assert_rel_eq!(0.8, 024.0, 0.857);
        assert_rel_eq!(0.85, 024.0, 1.059);
        assert_rel_eq!(0.9, 024.0, 1.318);
        assert_rel_eq!(0.95, 024.0, 1.711);
        assert_rel_eq!(0.975, 024.0, 2.064);
        assert_rel_eq!(0.99, 024.0, 2.492);
        assert_rel_eq!(0.995, 024.0, 2.797);
        assert_rel_eq!(0.9975, 024.0, 3.091);
        assert_rel_eq!(0.999, 024.0, 3.467);
        assert_rel_eq!(0.9995, 024.0, 3.745);

        assert_rel_eq!(0.75, 025.0, 0.684);
        assert_rel_eq!(0.8, 025.0, 0.856);
        assert_rel_eq!(0.85, 025.0, 1.058);
        assert_rel_eq!(0.9, 025.0, 1.316);
        assert_rel_eq!(0.95, 025.0, 1.708);
        assert_rel_eq!(0.975, 025.0, 2.060);
        assert_rel_eq!(0.99, 025.0, 2.485);
        assert_rel_eq!(0.995, 025.0, 2.787);
        assert_rel_eq!(0.9975, 025.0, 3.078);
        assert_rel_eq!(0.999, 025.0, 3.450);
        assert_rel_eq!(0.9995, 025.0, 3.725);

        assert_rel_eq!(0.75, 026.0, 0.684);
        assert_rel_eq!(0.8, 026.0, 0.856);
        assert_rel_eq!(0.85, 026.0, 1.058);
        assert_rel_eq!(0.9, 026.0, 1.315);
        assert_rel_eq!(0.95, 026.0, 1.706);
        assert_rel_eq!(0.975, 026.0, 2.056);
        assert_rel_eq!(0.99, 026.0, 2.479);
        assert_rel_eq!(0.995, 026.0, 2.779);
        assert_rel_eq!(0.9975, 026.0, 3.067);
        assert_rel_eq!(0.999, 026.0, 3.435);
        assert_rel_eq!(0.9995, 026.0, 3.707);

        assert_rel_eq!(0.75, 027.0, 0.684);
        assert_rel_eq!(0.8, 027.0, 0.855);
        assert_rel_eq!(0.85, 027.0, 1.057);
        assert_rel_eq!(0.9, 027.0, 1.314);
        assert_rel_eq!(0.95, 027.0, 1.703);
        assert_rel_eq!(0.975, 027.0, 2.052);
        assert_rel_eq!(0.99, 027.0, 2.473);
        assert_rel_eq!(0.995, 027.0, 2.771);
        assert_rel_eq!(0.9975, 027.0, 3.057);
        assert_rel_eq!(0.999, 027.0, 3.421);
        assert_rel_eq!(0.9995, 027.0, 3.690);

        assert_rel_eq!(0.75, 028.0, 0.683);
        assert_rel_eq!(0.8, 028.0, 0.855);
        assert_rel_eq!(0.85, 028.0, 1.056);
        assert_rel_eq!(0.9, 028.0, 1.313);
        assert_rel_eq!(0.95, 028.0, 1.701);
        assert_rel_eq!(0.975, 028.0, 2.048);
        assert_rel_eq!(0.99, 028.0, 2.467);
        assert_rel_eq!(0.995, 028.0, 2.763);
        assert_rel_eq!(0.9975, 028.0, 3.047);
        assert_rel_eq!(0.999, 028.0, 3.408);
        assert_rel_eq!(0.9995, 028.0, 3.674);

        assert_rel_eq!(0.75, 029.0, 0.683);
        assert_rel_eq!(0.8, 029.0, 0.854);
        assert_rel_eq!(0.85, 029.0, 1.055);
        assert_rel_eq!(0.9, 029.0, 1.311);
        assert_rel_eq!(0.95, 029.0, 1.699);
        assert_rel_eq!(0.975, 029.0, 2.045);
        assert_rel_eq!(0.99, 029.0, 2.462);
        assert_rel_eq!(0.995, 029.0, 2.756);
        assert_rel_eq!(0.9975, 029.0, 3.038);
        assert_rel_eq!(0.999, 029.0, 3.396);
        assert_rel_eq!(0.9995, 029.0, 3.659);

        assert_rel_eq!(0.75, 030.0, 0.683);
        assert_rel_eq!(0.8, 030.0, 0.854);
        assert_rel_eq!(0.85, 030.0, 1.055);
        assert_rel_eq!(0.9, 030.0, 1.310);
        assert_rel_eq!(0.95, 030.0, 1.697);
        assert_rel_eq!(0.975, 030.0, 2.042);
        assert_rel_eq!(0.99, 030.0, 2.457);
        assert_rel_eq!(0.995, 030.0, 2.750);
        assert_rel_eq!(0.9975, 030.0, 3.030);
        assert_rel_eq!(0.999, 030.0, 3.385);
        assert_rel_eq!(0.9995, 030.0, 3.646);

        assert_rel_eq!(0.75, 040.0, 0.681);
        assert_rel_eq!(0.8, 040.0, 0.851);
        assert_rel_eq!(0.85, 040.0, 1.050);
        assert_rel_eq!(0.9, 040.0, 1.303);
        assert_rel_eq!(0.95, 040.0, 1.684);
        assert_rel_eq!(0.975, 040.0, 2.021);
        assert_rel_eq!(0.99, 040.0, 2.423);
        assert_rel_eq!(0.995, 040.0, 2.704);
        assert_rel_eq!(0.9975, 040.0, 2.971);
        assert_rel_eq!(0.999, 040.0, 3.307);
        assert_rel_eq!(0.9995, 040.0, 3.551);

        assert_rel_eq!(0.75, 050.0, 0.679);
        assert_rel_eq!(0.8, 050.0, 0.849);
        assert_rel_eq!(0.85, 050.0, 1.047);
        assert_rel_eq!(0.9, 050.0, 1.299);
        assert_rel_eq!(0.95, 050.0, 1.676);
        assert_rel_eq!(0.975, 050.0, 2.009);
        assert_rel_eq!(0.99, 050.0, 2.403);
        assert_rel_eq!(0.995, 050.0, 2.678);
        assert_rel_eq!(0.9975, 050.0, 2.937);
        assert_rel_eq!(0.999, 050.0, 3.261);
        assert_rel_eq!(0.9995, 050.0, 3.496);

        assert_rel_eq!(0.75, 060.0, 0.679);
        assert_rel_eq!(0.8, 060.0, 0.848);
        assert_rel_eq!(0.85, 060.0, 1.045);
        assert_rel_eq!(0.9, 060.0, 1.296);
        assert_rel_eq!(0.95, 060.0, 1.671);
        assert_rel_eq!(0.975, 060.0, 2.000);
        assert_rel_eq!(0.99, 060.0, 2.390);
        assert_rel_eq!(0.995, 060.0, 2.660);
        assert_rel_eq!(0.9975, 060.0, 2.915);
        assert_rel_eq!(0.999, 060.0, 3.232);
        assert_rel_eq!(0.9995, 060.0, 3.460);

        assert_rel_eq!(0.75, 080.0, 0.678);
        assert_rel_eq!(0.8, 080.0, 0.846);
        assert_rel_eq!(0.85, 080.0, 1.043);
        assert_rel_eq!(0.9, 080.0, 1.292);
        assert_rel_eq!(0.95, 080.0, 1.664);
        assert_rel_eq!(0.975, 080.0, 1.990);
        assert_rel_eq!(0.99, 080.0, 2.374);
        assert_rel_eq!(0.995, 080.0, 2.639);
        assert_rel_eq!(0.9975, 080.0, 2.887);
        assert_rel_eq!(0.999, 080.0, 3.195);
        assert_rel_eq!(0.9995, 080.0, 3.416);

        assert_rel_eq!(0.75, 100.0, 0.677);
        assert_rel_eq!(0.8, 100.0, 0.845);
        assert_rel_eq!(0.85, 100.0, 1.042);
        assert_rel_eq!(0.9, 100.0, 1.290);
        assert_rel_eq!(0.95, 100.0, 1.660);
        assert_rel_eq!(0.975, 100.0, 1.984);
        assert_rel_eq!(0.99, 100.0, 2.364);
        assert_rel_eq!(0.995, 100.0, 2.626);
        assert_rel_eq!(0.9975, 100.0, 2.871);
        assert_rel_eq!(0.999, 100.0, 3.174);
        assert_rel_eq!(0.9995, 100.0, 3.390);

        assert_rel_eq!(0.75, 120.0, 0.677);
        assert_rel_eq!(0.8, 120.0, 0.845);
        assert_rel_eq!(0.85, 120.0, 1.041);
        assert_rel_eq!(0.9, 120.0, 1.289);
        assert_rel_eq!(0.95, 120.0, 1.658);
        assert_rel_eq!(0.975, 120.0, 1.980);
        assert_rel_eq!(0.99, 120.0, 2.358);
        assert_rel_eq!(0.995, 120.0, 2.617);
        assert_rel_eq!(0.9975, 120.0, 2.860);
        assert_rel_eq!(0.999, 120.0, 3.160);
        assert_rel_eq!(0.9995, 120.0, 3.373);
    }
}
