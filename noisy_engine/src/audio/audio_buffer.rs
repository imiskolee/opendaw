use num_traits::Float;
use std::ptr::null_mut;

/// zero-cost audio buffer, typically the raw data is created in the audio device driver
/// and the application holds the read and write pointers.
pub struct AudioBuffer<T: Float> {
    pub raw_ptr: *mut *mut T,
    pub data: Vec<Vec<T>>,
    pub sample_rates: f64,
    pub channels: usize,
    pub length: usize,
    allowed_extend: bool,
}

impl<T: Float> AudioBuffer<T> {
    pub fn new(sample_rates: f64, channels: usize) -> Self {
        unsafe {
            let mut data = vec![vec![]; channels];
            let mut raw_pointer: Vec<*mut T> = data.iter_mut().map(|e| e.as_mut_ptr()).collect();
            let buffer = Self { raw_ptr: raw_pointer.as_mut_ptr(), data: data, sample_rates: sample_rates, channels: channels, length: 0, allowed_extend:true };
            buffer
        }
    }

    pub fn from_raw_data(
        data: *mut *mut T,
        sample_rates: f64,
        channels: usize,
        length: usize,
    ) -> Self {
        let mut v: Vec<Vec<T>> = Vec::with_capacity(channels);
        for idx in 0..channels {
            unsafe {
                v.push(Vec::from_raw_parts(*data.add(idx), length, 0));
            }
        }
        let buffer = Self { raw_ptr: data, data: v, sample_rates: sample_rates, channels: channels, length: length,allowed_extend:false };
        buffer
    }

    pub fn length_seconds(&self) -> f64 {
        self.length as f64 / self.sample_rates
    }

    pub fn length_samples(&self) -> usize {
        self.length
    }

    pub fn get_reader(&self, channel: usize) -> &[T] {
        self.data[channel].as_slice()
    }

    pub fn get_writer(&mut self, channel: usize) -> &mut Vec<T> {
        self.data[channel].as_mut()
    }
    pub fn extend(&mut self,samples: usize) {
        assert!(self.allowed_extend,"create an audio buffer dependence outside memory,can not extend");
        self.length += samples;
        self.data.iter_mut().for_each(|e| e.resize(e.len() + samples,T::zero()));
    }

}

#[test]
fn test_audio_buffer() {
    let size: usize = 1024;
    let channels: usize = 2;
    let sample_rates = 44100.0;
    let mut empty_sample: Vec<Vec<f64>> = vec![vec![0.0f64; size]; channels];
    empty_sample[0][1] = 0.5;
    let mut raw_pointer: Vec<*mut f64> = empty_sample.iter_mut().map(|e| e.as_mut_ptr()).collect();
    let mut buffer =
        AudioBuffer::from_raw_data(raw_pointer.as_mut_ptr(), sample_rates, channels, size);

    assert_eq!(buffer.channels, channels);
    assert_eq!(buffer.length, size);
    assert_eq!(buffer.get_reader(0).len(), size);
    assert_eq!(buffer.get_writer(1).len(), size);
    assert_eq!(buffer.length_samples(), size);
    {
        assert_eq!(buffer.get_reader(0)[1], 0.5);
        buffer.get_writer(1)[0] = 1.0;
        assert_eq!(buffer.get_writer(1)[0], 1.0);
        assert_eq!(buffer.get_reader(1)[0], 1.0);
    }
    {
        let mut buffer:AudioBuffer<f64> = AudioBuffer::new(sample_rates,channels);
        buffer.extend(1024);
        assert_eq!(buffer.length_samples(),1024);
    }
}
