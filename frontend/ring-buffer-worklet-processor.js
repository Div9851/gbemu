import { RingBuffer } from "./ringbuf.js"

class RingBufferWorkletProcessor extends AudioWorkletProcessor {
    constructor(...args) {
        super(...args);
        this.ringBuffer = new RingBuffer(new ArrayBuffer(65536));
        this.port.onmessage = (e) => {
            this.ringBuffer.push(e.data);
        }
    }

    process(inputs, outputs, parameters) {
        const output = outputs[0];
        const channel = output[0];
        this.ringBuffer.pop(channel);
        return true;
    }
}

registerProcessor("ring-buffer-worklet-processor", RingBufferWorkletProcessor);
