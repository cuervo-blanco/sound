#include <iostream>
#include <fstream>
#include <vector>
#include <cstdint>
#include <cmath>
#include <fftw3.h>

bool readWAV(const std::string &filename, std::vector<double> &samples, uint32_t &sampleRate) {
    std::ifstream file(filename, std::ios::binary);
    if (!file) {
        std::cerr << "Unable to open WAV file!" << std::endl;
        return false;
    }

    struct RIFFHeader {
        char chunkID[4];
        uint32_t chunkSize;
        char format[4];
    } riffHeader;

    struct FMTSubchunk {
        char subchunk1ID[4];
        uint32_t subchunk1Size;
        uint16_t audioFormat;
        uint16_t numChannels;
        uint32_t sampleRate;
        uint32_t byteRate;
        uint16_t blockAlign;
        uint16_t bitsPerSample;
    } fmtSubchunk;

    struct DataSubchunk {
        char subchunk2ID[4];
        uint32_t subchunk2Size;
    } dataSubchunk;

    file.read(reinterpret_cast<char *>(&riffHeader), sizeof(RIFFHeader));
    file.read(reinterpret_cast<char *>(&fmtSubchunk), sizeof(FMTSubchunk));

    if (fmtSubchunk.subchunk1Size > 16) {
        file.seekg(fmtSubchunk.subchunk1Size - 16, std::ios::cur);
    }

    while(true) {
        file.read(reinterpret_cast<char *>(&dataSubchunk), sizeof(DataSubchunk));
        if (std::strncmp(dataSubchunk.subchunk2ID,"data", 4) == 0)
            break;
        file.seekg(dataSubchunk.subchunk2Size, std::ios::cur);
    }

    if (fmtSubchunk.audioFormat != 1) {
        std::cerr << "Only PCM WAV files are supported!" << std::endl;
        return false;
    }

    sampleRate = fmtSubchunk.sampleRate;
    uint16_t bitsPerSample = fmtSubchunk.bitsPerSample;
    uint16_t numChannels = fmtSubchunk.numChannels;
    uint32_t bytesPerSample = bitsPerSample / 8;
    uint32_t totalSamples = dataSubchunk.subchunk2Size / (bytesPerSample * numChannels);

    samples.resize(totalSamples);

    for (uint32_t i = 0; i < totalSamples; ++i) {
        double sampleValue = 0.0;
        for (uint16_t ch = 0; ch < numChannels; ++ch) {
            int32_t sample = 0;
            file.read(reinterpret_cast<char *>(&sample), bytesPerSample);

            if (bitsPerSample == 8) {
                sample -= 128;
            } else if (bitsPerSample > 8) {
                int32_t maxVal = 1 << (bitsPerSample - 1);
                if (sample >= maxVal) {
                    sample -= (1 << bitsPerSample);
                }
            }
            // Normalize sample to [-1.0, 1.0] and sum channels
            sampleValue += sample / static_cast<double>(1 << (bitsPerSample - 1));
        }
        // Average the channels
        samples[i] = sampleValue / numChannels;
    }

    file.close();
    return true;
}

void generateImage(const std::vector<double> &fftData, const std::string &filename) {
    const int desiredWidth = 800; // Set the desired width of the image
    const int height = 400;
    std::ofstream img(filename);

    // Resample fftData to fit into desiredWidth
    std::vector<double> resampledData(desiredWidth, 0.0);
    size_t dataSize = fftData.size();
    size_t binSize = dataSize / desiredWidth;

    for (int i = 0; i < desiredWidth; ++i) {
        size_t startIdx = i * binSize;
        size_t endIdx = startIdx + binSize;
        if (i == desiredWidth - 1) {
            endIdx = dataSize;
        }

        double sum = 0.0;
        for (size_t j = startIdx; j < endIdx; ++j) {
            sum += fftData[j];
        }
        resampledData[i] = sum / (endIdx - startIdx);
    }

    double maxVal = *std::max_element(resampledData.begin(), resampledData.end());
    std::cout << "Max FFT Magnitude Value: " << maxVal << std::endl;

    if (maxVal <= 0.0) {
        std::cerr << "Error: Max FFT magnitude is zero or negative. Cannot normalize data." << std::endl;
        return;
    }

    img << "P3\n" << desiredWidth << " " << height << "\n255\n";

    for (int y = 0; y < height; ++y) {
        for (int x = 0; x < desiredWidth; ++x) {
            double value = (resampledData[x] / maxVal) * height;
            if (height - y < value) {
                // Draw line
                img << "0 0 0 ";
            } else {
                // Background
                img << "255 255 255 ";
            }
        }
        img << "\n";
    }

    img.close();
}

int main(int argc, char *argv[]) {
    if (argc != 2) {
        std::cerr << "Usage: fft <input_wav_file>" << std::endl;
        return 1;
    }

    std::string inputFilename = argv[1];
    std::vector<double> samples;
    uint32_t sampleRate;

    if (!readWAV(inputFilename, samples, sampleRate)) {
        return 1;
    }

    std::cout << "Number of samples read: " << samples.size() << std::endl;
    std::cout << "Sample rate: " << sampleRate << std::endl;

    std::cout << "First few audio samples:" << std::endl;
    for (size_t i = 0; i < 10 && i < samples.size(); ++i) {
        std::cout << "Sample[" << i << "]: " << samples[i] << std::endl;
    }


    size_t N = samples.size();

    fftw_complex *in = (fftw_complex *)fftw_malloc(sizeof(fftw_complex) * N);
    fftw_complex *out = (fftw_complex *)fftw_malloc(sizeof(fftw_complex) * N);

    for (size_t i = 0; i < N; ++i) {
        in[i][0] = samples[i]; // Real part
        in[i][1] = 0.0;        // Imaginary part
    }

    fftw_plan plan = fftw_plan_dft_1d(N, in, out, FFTW_FORWARD, FFTW_ESTIMATE);
    fftw_execute(plan);

    std::vector<double> fftMagnitude(N / 2);
    for (size_t i = 0; i < N / 2; ++i) {
        // Explain this equation
        fftMagnitude[i] = sqrt(out[i][0] * out[i][0] + out[i][1] * out[i][1]);
    }

    std::cout << "First few FFT magnitude values:" << std::endl;
    for (size_t i = 0; i < 10 && i < fftMagnitude.size(); ++i) {
        std::cout << "fftMagnitude[" << i << "]: " << fftMagnitude[i] << std::endl;
    }


    std::string outputImage = inputFilename + "_fft_spectrum.ppm";
    generateImage(fftMagnitude, outputImage);

    fftw_destroy_plan(plan);
    fftw_free(in);
    fftw_free(out);

    std::cout << "FFT image generated: " << outputImage << std::endl;

    return 0;
}
