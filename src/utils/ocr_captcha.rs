use async_channel::{Receiver, Sender};
use image::DynamicImage;
use leptess::{LepTess, Variable};
use rayon::prelude::*;

use super::{cards_handler::Character, ocr_drop::sub_ocr};

pub async fn captcha_ocr_loop(captcha_receiver: Receiver<(DynamicImage, Sender<[Character; 1]>)>) {
    let mut workers: [_; 3] = std::array::from_fn(|_| {
        let mut worker = LepTess::new(None, "eng").unwrap();
        worker
            .set_variable(Variable::TesseditPagesegMode, "7")
            .unwrap();
        worker
    });
    loop {
        let (im, return_sender) = captcha_receiver.recv().await.unwrap();
        let output = ocr_captcha(&mut workers, &im);
        let card_arr: [Character; 1] = std::array::from_fn(|_| Character {
            name: output.get(0).unwrap().to_owned(),
            series: output.get(1).unwrap().to_owned(),
            gen: match output
                .get(2)
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
    &[18, 460, 290, 27],
    &[18, 488, 290, 27],
    &[41, 430, 108, 27],
];

fn ocr_captcha(workers: &mut [LepTess; 3], im: &DynamicImage) -> Vec<String> {
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
