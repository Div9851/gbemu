import { Server } from "socket.io"
import crypto from "crypto"

const io = new Server(3000, {
    cors: {
        origin: "http://localhost:1234",
        methods: ["GET", "POST"]
    }
});

io.on("connection", (socket) => {
    const userID = crypto.randomUUID();
    socket.join(userID);
    socket.onAny((eventName, arg) => {
        console.log(eventName, arg);
        const { to, from, data } = arg;
        io.to(to).emit(eventName, { from: from, data: data });
    });
    socket.emit("notify-friend-id", userID);
});
