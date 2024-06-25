import { RingBuffer } from "./ringbuf"

class RingBufferWorkletProcessor extends AudioWorkletProcessor {
    ringBuffer: RingBuffer
    constructor() {
        super();
        this.ringBuffer = new RingBuffer(new ArrayBuffer(65536));
        this.port.onmessage = (e) => {
            this.ringBuffer.push(e.data);
        }
    }

    process(input: Float32Array[][], outputs: Float32Array[][], parameters: Record<string, Float32Array>): boolean {
        const output = outputs[0];
        const channel = output[0];
        this.ringBuffer.pop(channel);
        return true;
    }
}

registerProcessor("ring-buffer-worklet-processor", RingBufferWorkletProcessor);
