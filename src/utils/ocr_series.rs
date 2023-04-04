use async_channel::{Receiver, Sender};
use image::DynamicImage;
use leptess::{LepTess, Variable};
use rayon::prelude::*;

use super::{cards_handler::Character, ocr_drop::sub_ocr};

pub async fn series_ocr_loop(series_receiver: Receiver<(DynamicImage, Sender<[Character; 2]>)>) {
    let mut workers: [_; 6] = std::array::from_fn(|_| {
        let mut worker = LepTess::new(None, "eng").unwrap();
        worker
            .set_variable(Variable::TesseditPagesegMode, "7")
            .unwrap();
        worker
    });
    loop {
        let (im, return_sender) = series_receiver.recv().await.unwrap();
        let output = ocr_series(&mut workers, &im);
        let card_arr: [Character; 2] = std::array::from_fn(|i| Character {
            name: output.get(i * 2).unwrap().to_owned(),
            series: output.get(i * 2 + 1).unwrap().to_owned(),
            gen: match output
                .get(6 + i * 2 / 2)
                .unwrap_or(&"0".to_string())
                .to_owned()
                .parse::<u16>()
                .unwrap()
            {
                0 => None,
                other => Some(other),
            },
            wl: None,
        });
        return_sender.send(card_arr).await.unwrap();
    }
}

static CORDS_GEN: &[&[u32]] = &[
    &[26, 457, 287, 27],
    &[26, 486, 287, 27],
    &[379, 457, 287, 27],
    &[379, 486, 287, 27],
    &[50, 427, 108, 27],
    &[403, 427, 108, 27],
];

fn ocr_series(workers: &mut [LepTess; 6], im: &DynamicImage) -> Vec<String> {
    let arr = workers
        .par_iter_mut()
        .enumerate()
        .map(|(i, worker)| {
            if CORDS_GEN[i][2] == 108 {
                worker
                    .set_variable(Variable::TesseditCharWhitelist, "1234567890")
                    .unwrap();
            } else {
                worker
                    .set_variable(Variable::TesseditCharWhitelist, "")
                    .unwrap();
            }
            sub_ocr(
                &mut im.clone(),
                worker,
                CORDS_GEN[i][0],
                CORDS_GEN[i][1],
                CORDS_GEN[i][2],
                CORDS_GEN[i][3],
            )
        })
        .collect();
    arr
}
