use crate::algorithms::ellipse::ellipse_base;
use crate::algorithms::ellipse::ellipse_base::EllipseBase;
use crate::algorithms::{Algorithm, Point};

pub struct OwnDecisionEllipse {
    pub base: EllipseBase,
}

impl Default for OwnDecisionEllipse {
    fn default() -> Self {
        let mut s = Self {
            base: EllipseBase::new(0, 0, 30, 20),
        };
        s.compute();
        s
    }
}

impl Algorithm for OwnDecisionEllipse {
    fn name(&self) -> &str {
        "Own Decision Ellipse"
    }

    fn compute(&mut self) {
        self.base.points.clear();
        self.base.point_rings.clear();

        let max_ry = self.base.radius_y;
        for i in 0..=max_ry {
            let rx = self.base.radius_x - i;
            let ry = self.base.radius_y - i;
            if rx >= 1 && ry >= 1 {
                self.compute_ring(rx, ry, i + 1);
            }
        }
        // middle horizontal line to fill the center
        let final_rx = self.base.radius_x - (self.base.radius_y - 1);
        for x in -final_rx..=final_rx {
            self.base.points.push(Point { x, y: 0 });
            self.base.point_rings.push(max_ry + 1);
        }
    }

    fn points(&self) -> &[Point] {
        &self.base.points
    }

    fn color(&self) -> [f32; 3] {
        [0.2, 0.2, 0.38]
    }

    fn point_color_override(&self, i: usize) -> Option<[f32; 3]> {
        let fallback = {
            let r = *self.base.point_rings.get(i)? as usize;
            Some(ellipse_base::PALETTE_7[r % 7])
        };
        self.base.duplicate_color(i, fallback)
    }

    fn overlay_ellipse(&self) -> Option<(i32, i32, i32, i32)> {
        self.base.overlay_ellipse_base()
    }

    fn cell_info(&self, cx: i32, cy: i32) -> Option<String> {
        self.base.cell_info_ring_base(cx, cy)
    }

    fn draw_ui(&mut self, ui: &mut egui::Ui) -> bool {
        self.base.draw_ui_full(ui)
    }

    fn category(&self) -> &'static str {
        "ellipse"
    }
}

impl OwnDecisionEllipse {
    fn compute_ring(&mut self, rx: i32, ry: i32, ring: i32) {
        let mut x = 0i32;
        let mut y = ry;

        // Region 1 alap döntési változója
        let mut d1 = (ry * ry - rx * rx * ry) as f64 + 0.25 * (rx * rx) as f64;
        let mut dx = 2 * ry * ry * x;
        let mut dy = 2 * rx * rx * y;

        // --- GAP / HIÁNYZÓ PIXEL KITÖLTÉS ELŐKÉSZÍTÉSE ---
        // A levezetésed szerinti kezdő Delta_d (a=rx, b=ry):
        // Delta_d = b - 2ab - a^2 - 1.5*a
        // Megjegyzés: A levezetés előjeleitől függően ez a külső és belső közötti eltérést méri.
        let mut d_gap_1 =
            ry as f64 - 2.0 * rx as f64 * ry as f64 - (rx * rx) as f64 - 1.5 * rx as f64;

        // Region 1
        while dx < dy {
            self.base.plot_symmetric(x, y, ring);

            if d1 < 0.0 {
                // Vízszintes lépés az alap ellipszisen
                x += 1;
                dx += 2 * ry * ry;
                d1 += dx as f64 + (ry * ry) as f64;

                // Inkrementális frissítés vízszintes lépés esetén (Levezetés 6. pontja alapján)
                // Delta_new = Delta + (2b + 1)*(2x + 1)
                // Figyelem: mivel az x már növekedett, az inkrementális tagot ehhez igazítjuk
                d_gap_1 += (2 * ry + 1) as f64 * (2 * x + 1) as f64;
            } else {
                // Diagonális lépés az alap ellipszisen
                x += 1;
                y -= 1;
                dx += 2 * ry * ry;
                dy -= 2 * rx * rx;
                d1 += (dx - dy + ry * ry) as f64;

                // Inkrementális frissítés diagonális lépés esetén (Levezetés 7. pontja alapján)
                // Delta_new = Delta + (2b + 1)*(2x + 1) + 2*(2a + 1)*y
                d_gap_1 +=
                    (2 * ry + 1) as f64 * (2 * x + 1) as f64 + 2.0 * (2 * rx + 1) as f64 * y as f64;
            }

            // --- GAP ELLENŐRZÉS ÉS PIXEL KITÖLTÉS ---
            // Ha a gap döntési változó átlép egy kritikus aszinkronitási küszöböt,
            // az azt jelenti, hogy a belső/külső ív elvált egymástól, és lyuk keletkezne.
            // Itt egy korrekciós (szomszédos) pixelt rajzolunk ki.
            if d_gap_1.abs() > (ry * ry) as f64 {
                // Kirajzoljuk a hiányzó átlós/szomszédos pixelt
                self.base.plot_symmetric(x, y + 1, ring);
                // Korrigáljuk a gap változót, hogy ne duplázzon feleslegesen
                d_gap_1 -= (ry * ry) as f64 * d_gap_1.signum();
            }
        }

        // Region 2 kezdőértéke az alap ellipszishez
        let mut d2 = (ry * ry) as f64 * (x as f64 + 0.5) * (x as f64 + 0.5)
            + (rx * rx) as f64 * (y - 1) as f64 * (y - 1) as f64
            - (rx * rx) as f64 * (ry * ry) as f64;

        // Region 2 gap változó inicializálása a Region 1 végén lévő állapotból
        let mut d_gap_2 = d_gap_1;

        // Region 2
        while y >= 0 {
            self.base.plot_symmetric(x, y, ring);

            if d2 > 0.0 {
                // Függőleges lépés
                y -= 1;
                dy -= 2 * rx * rx;
                d2 += (rx * rx) as f64 - dy as f64;

                // Region 2-ben a lépésirány megváltozik (y csökken),
                // így a differenciál tag is az y változásához igazodik
                d_gap_2 -= (2 * rx + 1) as f64 * (2 * y + 1) as f64;
            } else {
                // Diagonális lépés
                y -= 1;
                x += 1;
                dx += 2 * ry * ry;
                dy -= 2 * rx * rx;
                d2 += (dx - dy + rx * rx) as f64;

                d_gap_2 += (2 * ry + 1) as f64 * (2 * x + 1) as f64
                    - (2 * rx + 1) as f64 * (2 * y + 1) as f64;
            }

            // --- GAP ELLENŐRZÉS REGION 2-BEN ---
            if d_gap_2.abs() > (rx * rx) as f64 {
                // Ha a Region 2-ben keletkezik rés, vízszintes irányú korrekciót alkalmazunk
                self.base.plot_symmetric(x - 1, y, ring);
                d_gap_2 -= (rx * rx) as f64 * d_gap_2.signum();
            }
        }
    }
}
