import wasmInit, { Emulator, InitOutput, JoypadInput } from "./wasm/gbemu_core"
// @ts-ignore
import workletUrl from "worklet:./ring-buffer-worklet-processor"
import { io, Socket } from "socket.io-client"

let wasm: InitOutput | null = null;
let emulator: Emulator | null = null;
let audioCtx: AudioContext | null = null;
let ringBufferNode: AudioWorkletNode | null = null;
let socket: Socket | null = null;
let myID: string | null = null;
let opponentID: string | null = null;
let isReady = false;

const screen = document.getElementById("screen") as HTMLCanvasElement;
const romInput = document.getElementById("rom") as HTMLInputElement;
const savedataInput = document.getElementById("savedata") as HTMLInputElement;
const friendIdInput = document.getElementById("friend-id") as HTMLInputElement;
const selectRom = document.getElementById("select-rom") as HTMLButtonElement;
const importSavedata = document.getElementById("import-savedata") as HTMLButtonElement;
const exportSavedata = document.getElementById("export-savedata") as HTMLButtonElement;
const joinServer = document.getElementById("join-server") as HTMLButtonElement;
const searchFriend = document.getElementById("search-friend") as HTMLButtonElement;

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
    if (isReady) {
        emulator.send_data((data: number) => {
            socket?.emit("data-transfer", {
                from: myID,
                to: opponentID,
                data: data,
            });
        });
    } else {
        emulator.send_data((data: number) => {
            console.log(`serial data: 0x${data}`);
        });
    }
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
    audioCtx = new AudioContext({ sampleRate: 4194304 / 87 });
    await audioCtx.audioWorklet.addModule(workletUrl);
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

const joinServerHandler = () => {
    if (socket != null) {
        alert("You already joined the game server");
        return;
    }
    socket = io("ws://localhost:3000");

    socket.on("connect", () => {
        console.log("Joined the game server");
    });

    socket.on("notify-friend-id", (arg) => {
        myID = arg;
        alert(`Your friend ID is ${arg}`);
        console.log(`Your friend ID is ${arg}`);
    });

    socket.on("connection-request", ({ from }: { from: string }) => {
        opponentID = from;
        console.log(`received connection request from ${opponentID}`);
        socket?.emit("connection-request-ack", {
            from: myID,
            to: opponentID,
        });
        isReady = true;
    });

    socket.on("connection-request-ack", ({ from }: { from: string }) => {
        console.log(`connection request to ${from} has been accepted`);
        isReady = true;
    });

    socket.on("data-transfer", ({ from, data }: { from: string, data: number }) => {
        if (emulator == null || from !== opponentID) {
            return;
        }
        console.log(`receive 0x${data.toString(16)} from ${opponentID}`)
        const prev = emulator.receive_data(data);
        socket?.emit("data-transfer-reply", { from: myID, to: opponentID, data: prev });
    });

    socket.on("data-transfer-reply", ({ from, data }: { from: string, data: number }) => {
        if (emulator == null || from !== opponentID) {
            return;
        }
        console.log(`receive 0x${data.toString(16)} from ${opponentID}`)
        emulator.receive_data(data);
    });
}

const searchFriendHandler = () => {
    opponentID = friendIdInput.value;
    console.log(`sent connection request to ${opponentID}`)
    socket?.emit("connection-request", {
        from: myID,
        to: opponentID,
    });
}

romInput.addEventListener("change", romInputChangeHandler);
savedataInput.addEventListener("change", savedataInputChangeHandler);
selectRom.addEventListener("click", () => romInput.click());
importSavedata.addEventListener("click", () => savedataInput.click());
exportSavedata.addEventListener("click", exportSavedataHandler);
joinServer.addEventListener("click", joinServerHandler);
searchFriend.addEventListener("click", searchFriendHandler);

const init = async () => {
    wasm = await wasmInit();
    emulator = new Emulator();
}

init();
