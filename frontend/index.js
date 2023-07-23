var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
import wasmInit, { Emulator, JoypadInput } from "./wasm/gbemu_core.js";
let wasm = null;
let emulator = null;
const screen = document.getElementById("screen");
const romInput = document.getElementById("rom");
const getMemorySliceAsUint8Array = (start, length) => {
    const buf = new Uint8Array(wasm.memory.buffer);
    return buf.slice(start, start + length);
};
const render = (canvas, imageDataArray) => {
    const canvasContext = canvas.getContext("2d");
    const imageData = new ImageData(canvas.width, canvas.height);
    imageData.data.set(imageDataArray);
    canvasContext.putImageData(imageData, 0, 0);
};
let upPressed = false;
let downPressed = false;
let leftPressed = false;
let rightPressed = false;
let buttonAPressed = false;
let buttonBPressed = false;
let selectPressed = false;
let startPressed = false;
const keyDownHandler = (event) => {
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
};
const keyUpHandler = (event) => {
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
};
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
};
const romInputChangeHandler = () => __awaiter(void 0, void 0, void 0, function* () {
    if (emulator == null) {
        return;
    }
    if (romInput.files == null || romInput.files.length === 0) {
        return;
    }
    const romFile = romInput.files[0];
    const buf = yield romFile.arrayBuffer();
    const romData = new Uint8Array(buf);
    emulator.init();
    emulator.load_rom(romData);
    emulator.run();
    requestAnimationFrame(nextFrame);
});
romInput.addEventListener("change", romInputChangeHandler);
const runWasm = () => __awaiter(void 0, void 0, void 0, function* () {
    wasm = yield wasmInit();
    emulator = new Emulator();
});
runWasm();
export { emulator };
