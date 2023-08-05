import wasmInit, { Emulator, InitOutput, JoypadInput } from "./wasm/gbemu_core.js"

let wasm: InitOutput | null = null;
let emulator: Emulator | null = null;
let audioCtx: AudioContext | null = null;
let ringBufferNode: AudioWorkletNode | null = null;

const screen = document.getElementById("screen") as HTMLCanvasElement;
const romInput = document.getElementById("rom") as HTMLInputElement;
const savedataInput = document.getElementById("savedata") as HTMLInputElement;
const selectRom = document.getElementById("select-rom") as HTMLButtonElement;
const importSavedata = document.getElementById("import-savedata") as HTMLButtonElement;
const exportSavedata = document.getElementById("export-savedata") as HTMLButtonElement;

let prevTime: number | null = null;
const targetPeriod: number = 1000 / 60;

const getMemorySliceAsUint8Array = (start: number, length: number): Uint8Array => {
    const buf = new Uint8Array(wasm!.memory.buffer);
    return buf.slice(start, start + length);
}

const render = (screen: HTMLCanvasElement, imageDataArray: Uint8Array) => {
    const screenContext = screen.getContext("2d");
    const imageData = new ImageData(160, 144);
    imageData.data.set(imageDataArray);
    createImageBitmap(imageData, 0, 0, imageData.width, imageData.height).then((bitmap) => {
        screenContext?.drawImage(bitmap, 0, 0, screen.width, screen.height);
    });
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

const nextFrame = (currentTime: number) => {
    if (emulator == null || !emulator.running) {
        return;
    }
    if (prevTime == null) {
        prevTime = currentTime;
    }
    const elapsedTime = currentTime - prevTime;
    if (elapsedTime + 1 < targetPeriod) {
        requestAnimationFrame(nextFrame);
        return;
    }
    emulator.update_joypad_input(JoypadInput.new(startPressed, selectPressed, buttonAPressed, buttonBPressed, downPressed, upPressed, leftPressed, rightPressed));
    emulator.next_frame();
    render(screen, emulator.get_frame_buffer());
    ringBufferNode?.port.postMessage(emulator.get_audio_buffer());
    prevTime = currentTime;
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
    audioCtx = new AudioContext({ sampleRate: 48000 });
    await audioCtx.audioWorklet.addModule("ring-buffer-worklet-processor.js");
    ringBufferNode = new AudioWorkletNode(
        audioCtx,
        "ring-buffer-worklet-processor",
    );
    ringBufferNode.connect(audioCtx.destination);
    requestAnimationFrame(nextFrame);
}

const savedataInputChangeHandler = async () => {
    if (emulator == null) {
        return;
    }
    if (savedataInput.files == null || savedataInput.files.length === 0) {
        return;
    }
    const savedataFile = savedataInput.files[0];
    const buf = await savedataFile.arrayBuffer();
    const savedata = new Uint8Array(buf);
    emulator.load_savedata(savedata);
}

const exportSavedataHandler = () => {
    if (emulator == null) {
        return;
    }
    const savedata = emulator.get_savedata();
    const blob = new Blob([savedata], { type: "binary/octet-stream" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a") as HTMLAnchorElement;
    document.body.appendChild(a);
    a.href = url;
    a.click();
}

romInput.addEventListener("change", romInputChangeHandler);
savedataInput.addEventListener("change", savedataInputChangeHandler);
selectRom.addEventListener("click", () => romInput.click());
importSavedata.addEventListener("click", () => savedataInput.click());
exportSavedata.addEventListener("click", exportSavedataHandler);

const init = async () => {
    wasm = await wasmInit();
    emulator = new Emulator();
}

init();
