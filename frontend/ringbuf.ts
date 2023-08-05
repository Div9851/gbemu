class RingBuffer {
    buffer: Float32Array;
    readPos: number;
    writePos: number;

    constructor(buffer: ArrayBuffer) {
        this.buffer = new Float32Array(buffer);
        this.readPos = 0;
        this.writePos = 0;
    }

    push(input: Float32Array) {
        for (let i = 0; i < input.length; i++) {
            this.buffer[this.writePos] = input[i];
            this.writePos = (this.writePos + 1) % this.buffer.length;
        }
    }

    pop(output: Float32Array) {
        const size = this.size();
        if (size < 128) return;
        for (let i = 0; i < 128; i++) {
            output[i] = this.buffer[this.readPos];
            this.readPos = (this.readPos + 1) % this.buffer.length;
        }
    }

    size() {
        return (this.writePos - this.readPos + this.buffer.length) % this.buffer.length;
    }
}

export { RingBuffer };
