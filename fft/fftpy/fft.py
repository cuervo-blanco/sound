import sys
import numpy as np
import matplotlib.pyplot as plt
from scipy.io import wavfile


def read_wav(filename):
    try:
        sample_rate, data = wavfile.read(filename)
        print(f"Sample rate: {sample_rate} Hz")
        print(f"Data type: {data.dtype}")
        print(f"Number of samples: {data.shape[0]}")

        if data.ndim > 1:
            data = data.mean(axis=1)
            print("Converted to mono by averaging channels.")

        if data.dtype == np.int16:
            data = data.astype(np.float32) / 32768.0
        elif data.dtype == np.int32:
            data = data.astype(np.float32) / 2147483648.0
        elif data.dtype == np.uint8:
            data = (data.astype(np.float32) - 128) / 128.0
        else:
            data = data.astype(np.float32)

        return sample_rate, data
    except Exception as e:
        print(f"Error reading WAV file: {e}")
        sys.exit(1)


def compute_fft(data):
    N = len(data)
    fft_data = np.fft.fft(data)
    fft_magnitude = np.abs(fft_data)[:N // 2]  # Take positive frequencies
    return fft_magnitude


def generate_image(fft_magnitude, output_filename):
    desired_width = 800
    height = 400

    data_size = len(fft_magnitude)
    resampled_data = np.interp(
        np.linspace(0, data_size, desired_width),
        np.arange(data_size),
        fft_magnitude
    )

    max_val = np.max(resampled_data)
    if max_val <= 0:
        print("Error: Max FFT magnitude is zero or negative.")
        sys.exit(1)
    normalized_data = resampled_data / max_val

    plt.figure(figsize=(desired_width / 100, height / 100), dpi=100)
    plt.plot(normalized_data, color='black')
    plt.fill_between(range(desired_width), normalized_data, color='black')
    plt.axis('off')
    plt.tight_layout()
    plt.savefig(output_filename, bbox_inches='tight', pad_inches=0)
    plt.close()
    print(f"FFT image generated: {output_filename}")


def main():
    if len(sys.argv) != 2:
        print("Usage: python fft.py <input_wav_file>")
        sys.exit(1)

    input_filename = sys.argv[1]
    output_filename = input_filename + "_fft_spectrum.png"

    sample_rate, data = read_wav(input_filename)
    fft_magnitude = compute_fft(data)
    generate_image(fft_magnitude, output_filename)


if __name__ == "__main__":
    main()
