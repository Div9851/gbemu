import wasmInit, { Emulator, InitOutput, JoypadInput } from "./wasm/gbemu_core.js"

let wasm: InitOutput | null = null;
let emulator: Emulator | null = null;

const screen = document.getElementById("screen") as HTMLCanvasElement;
const romInput = document.getElementById("rom") as HTMLInputElement;

const getMemorySliceAsUint8Array = (start: number, length: number): Uint8Array => {
    const buf = new Uint8Array(wasm!.memory.buffer);
    return buf.slice(start, start + length);
}

const render = (canvas: HTMLCanvasElement, imageDataArray: Uint8Array) => {
    const canvasContext = canvas.getContext("2d");
    const imageData = new ImageData(canvas.width, canvas.height);
    imageData.data.set(imageDataArray);
    canvasContext!.putImageData(imageData, 0, 0);
}

let upPressed = false;
let downPressed = false;
let leftPressed = false;
let rightPressed = false;
let buttonAPressed = false;
let buttonBPressed = false;
let selectPressed = false;
let startPressed = false;

const keyDownHandler = (event: KeyboardEvent) => {
    switch (event.key) {
        case "ArrowUp":
            upPressed = true;
            break;
        case "ArrowDown":
            downPressed = true;
            break;
        case "ArrowLeft":
            leftPressed = true;
            break;
        case "ArrowRight":
            rightPressed = true;
            break;
        case "x":
            buttonAPressed = true;
            break;
        case "z":
            buttonBPressed = true;
            break;
        case "Backspace":
            selectPressed = true;
            break;
        case "Enter":
            startPressed = true;
            break;
    }
}

const keyUpHandler = (event: KeyboardEvent) => {
    switch (event.key) {
        case "ArrowUp":
            upPressed = false;
            break;
        case "ArrowDown":
            downPressed = false;
            break;
        case "ArrowLeft":
            leftPressed = false;
            break;
        case "ArrowRight":
            rightPressed = false;
            break;
        case "x":
            buttonAPressed = false;
            break;
        case "z":
            buttonBPressed = false;
            break;
        case "Backspace":
            selectPressed = false;
            break;
        case "Enter":
            startPressed = false;
            break;
    }
}

document.addEventListener("keydown", keyDownHandler);
document.addEventListener("keyup", keyUpHandler);

const nextFrame = () => {
    if (emulator == null || !emulator.running) {
        return;
    }
    emulator.update_joypad_input(JoypadInput.new(startPressed, selectPressed, buttonAPressed, buttonBPressed, downPressed, upPressed, leftPressed, rightPressed));
    emulator.next_frame();
    render(screen, emulator.get_frame_buffer());
    requestAnimationFrame(nextFrame);

}

const romInputChangeHandler = async () => {
    if (emulator == null) {
        return;
    }
    if (romInput.files == null || romInput.files.length === 0) {
        return;
    }
    const romFile = romInput.files[0];
    const buf = await romFile.arrayBuffer();
    const romData = new Uint8Array(buf);
    emulator.init();
    emulator.load_rom(romData);
    emulator.run();
    requestAnimationFrame(nextFrame);
}

romInput.addEventListener("change", romInputChangeHandler);

const runWasm = async () => {
    wasm = await wasmInit();
    emulator = new Emulator();
}

runWasm();

export { emulator }
