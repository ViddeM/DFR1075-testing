use embassy_time::Duration;
use esp_hal::{
    dma::{DmaRxBuf, DmaTxBuf},
    gpio::Output,
    spi::master::{SpiDma, SpiDmaBus},
    Async,
};

// TODO: might not need these
const BLACK: u8 = 0;
const WHITE: u8 = 1;

const WIDTH: usize = 160;
const HEIGHT: usize = 68;

// const SHARPMEM_BIT_WRITECMD: u8 = 0x01; // 0x80 in LSB format
// const SHARPMEM_BIT_VCOM: u8 = 0x02; // 0x40 in LSB format
// const SHARPMEM_BIT_CLEAR: u8 = 0x04; // 0x20 in LSB format

// NOTE: flip these around if running in LSB mode
const SHARPMEM_BIT_WRITECMD: u8 = 0x80;
const SHARPMEM_BIT_VCOM: u8 = 0x40;
const SHARPMEM_BIT_CLEAR: u8 = 0x20;

pub struct NiceView {
    spi: SpiDmaBus<'static, Async>,
    cs: Output<'static>,
    vcom: u8,
}

impl NiceView {
    pub fn new(spi: SpiDma<'static, Async>, cs: Output<'static>) -> Self {
        let (rx_buf, rx_descriptors, tx_buf, tx_descriptors) = esp_hal::dma_buffers!(256, 256);

        let tx_buf = DmaTxBuf::new(tx_descriptors, tx_buf).unwrap();
        let rx_buf = DmaRxBuf::new(rx_descriptors, rx_buf).unwrap();
        let spi = spi.with_buffers(rx_buf, tx_buf);

        NiceView {
            spi,
            cs,
            vcom: SHARPMEM_BIT_VCOM,
        }
    }

    pub async fn clear_display(&mut self) {
        // TODO: what does the VCOM bit do??
        self.write(&[self.vcom | SHARPMEM_BIT_CLEAR, 0x00]).await;
        self.toggle_vcom();
    }

    pub async fn testy(&mut self) {
        const PIXELS: usize = WIDTH * HEIGHT;
        const BYTES: usize = PIXELS / 8;

        let mut buf = [0u8; BYTES + 2];

        buf[0] = SHARPMEM_BIT_WRITECMD;
        for n in 1..=BYTES {
            buf[n] = 0xff;
            buf[n + 1] = 0;

            let cmd = &buf[..=n + 1];

            self.cs.set_high();
            self.spi.write_async(cmd).await.unwrap();
            self.cs.set_low();

            embassy_time::Timer::after(Duration::from_millis(500)).await;
        }
    }

    pub async fn draw_test(&mut self, asdf: bool) {
        self.cs.set_high();

        // TODO: what does the VCOM bit do??
        self.spi
            .write_async(&[self.vcom | SHARPMEM_BIT_WRITECMD])
            .await
            .unwrap();
        self.toggle_vcom();

        //  uint8_t bytes_per_line = WIDTH / 8;
        //  uint16_t totalbytes = (WIDTH * HEIGHT) / 8;
        const BYTES_PER_LINE: usize = WIDTH / 8;
        let total_bytes = (WIDTH * HEIGHT) / 8;

        //  for (i = 0; i < totalbytes; i += bytes_per_line) {
        //    uint8_t line[bytes_per_line + 2];
        let all_black = 0x00;
        let all_white = 0xff;

        let (a, b) = if asdf {
            (all_black, all_white)
        } else {
            (all_white, all_black)
        };

        for i in (0..total_bytes).step_by(BYTES_PER_LINE) {
            const LEN: usize = BYTES_PER_LINE + 2;
            let mut line = [b; LEN];
            //let mut line = [0u8; { BYTES_PER_LINE + 2}];

            for b in &mut line[(LEN / 2)..] {
                *b = a;
            }

            // Send address byte
            let current_line = ((i + 1) / (WIDTH / 8)) as u8 + 1;
            line[0] = current_line;

            // TODO: actually send image data

            // End of line
            line[BYTES_PER_LINE + 1] = 0;

            self.spi.write_async(&line).await.unwrap();
        }

        // Send another trailing 8 bits for the last line
        self.spi.write_async(&[0x00]).await.unwrap();

        self.cs.set_low();
    }

    fn toggle_vcom(&mut self) {
        // ported directly adafruit driver
        // TODO: wtf is this shit?
        if self.vcom == 0 {
            self.vcom = SHARPMEM_BIT_VCOM
        } else {
            self.vcom = 0;
        }
    }

    pub async fn write(&mut self, data: &[u8]) {
        self.cs.set_high();
        self.spi.write_async(data).await.unwrap();
        self.cs.set_low();
    }
}
