// Execute the real movement callers, collision wrappers and route handoff.
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
import java.util.Arrays;

public class ZNativeDodgeMovementOracle extends ZNativeReactionsOracle {
    private static final long HW = 0x612f80, NODE = 0x5eee28, TERRAIN = 0x58aaaa;
    private static final int[][] DIRECTIONS = {
        {13, 0}, {3, 13}, {-13, 0}, {-3, 13},
        {13, -3}, {0, -13}, {-13, -3}, {-3, -13}
    };

    private void call(EmulatorHelper e, long entry, long ecx, long edx, long... args) throws Exception {
        put(e, S, 4, RET);
        for (int n = 0; n < args.length; n++) put(e, S + 4 + n * 4, 4, args[n]);
        e.writeRegister("ESP", S);
        e.writeRegister("ECX", ecx);
        e.writeRegister("EDX", edx);
        if (!e.run(toAddr(entry), null, monitor)) throw new Exception(e.getLastError());
        if (e.readRegister("EIP").longValue() != RET) throw new Exception("Unexpected stop");
    }

    private long[] fields(EmulatorHelper e) {
        return new long[] {
            (short)read(e, U + 0x2c, 2), (short)read(e, U + 0x2e, 2), read(e, U + 0x35, 1),
            read(e, U + 0x64, 1), read(e, U + 0x48, 2), read(e, U + 0x81, 1),
            read(e, U + 0x38, 1), read(e, U + 0x39, 1), read(e, U + 0x8c, 1),
            read(e, HW + 0x17, 1), read(e, U + 0x7e, 1), read(e, U + 0xbc, 1),
            read(e, U + 0x66, 1), read(e, U + 0x67, 1), read(e, U + 0x14, 4) == NODE ? 1 : 0,
            (short)read(e, U + 0x32, 2), (byte)read(e, U + 0x3c, 1)
        };
    }

    @Override public void run() throws Exception {
        try (PrintWriter out = new PrintWriter(getScriptArgs()[0])) {
            EmulatorHelper e = new EmulatorHelper(currentProgram);
            try {
                e.setBreakpoint(toAddr(RET));
                int scenario = 0;
                for (int cls = 0; cls < 2; cls++)
                for (int path = 0; path < 2; path++)
                for (int blocked = 0; blocked < 2; blocked++)
                for (int speed : new int[] { 1, 4 })
                for (int[] direction : DIRECTIONS) {
                    e.writeMemory(toAddr(U), new byte[192]);
                    e.writeMemory(toAddr(HW), new byte[44]);
                    e.writeMemory(toAddr(G), new byte[72]);
                    e.writeMemory(toAddr(NODE), new byte[8]);
                    byte[] terrain = new byte[16384];
                    if (blocked != 0) Arrays.fill(terrain, (byte)19);
                    e.writeMemory(toAddr(TERRAIN), terrain);
                    put(e, 0x622dc4, 1, 0);
                    put(e, 0x622c7a, 2, 0);
                    put(e, 0x625eae, 1, 0);
                    call(e, 0x433b30, 0, 0);
                    put(e, 0x632194, 4, G);
                    put(e, 0x622dc0, 1, 0);
                    put(e, 0x4d02a3, 1, 0);
                    put(e, U + 0x08, 4, cls == 1 ? HW : -1);
                    put(e, U + 0x2c, 2, 513);
                    put(e, U + 0x2e, 2, 513);
                    put(e, U + 0x30, 1, cls == 1 ? 34 : 4);
                    put(e, U + 0x31, 1, 4);
                    put(e, U + 0x34, 1, 255);
                    put(e, U + 0x35, 1, cls == 1 ? 58 : 1);
                    put(e, U + 0x38, 1, 68);
                    put(e, U + 0x3c, 1, 2);
                    put(e, U + 0x64, 1, 64);
                    put(e, U + 0x66, 1, 5);
                    put(e, U + 0x67, 1, 7);
                    put(e, U + 0x7c, 1, cls);
                    put(e, U + 0x8b, 1, 7);
                    put(e, U + 0x8c, 1, 2);
                    put(e, U + 0xbc, 1, 1);
                    if (path != 0) {
                        put(e, U + 0x14, 4, NODE);
                        put(e, NODE, 2, 760);
                        put(e, NODE + 2, 2, 652);
                    }
                    call(e, 0x41b950, U, 0);
                    call(e, 0x4525a0, U, 513 + direction[0], 513 + direction[1]);
                    for (int tick = 0; tick <= 16; tick++) {
                        int step = tick == 0 ? 0 : speed;
                        put(e, U + 0x62, 1, step);
                        if (cls == 0) call(e, 0x452200, 0, 0, U);
                        else call(e, 0x4502f0, U, 0);
                        out.printf("{\"scenario\":%d,\"tick\":%d,\"step\":%d,\"fields\":%s}%n",
                            scenario, tick, step, Arrays.toString(fields(e)));
                        if ((read(e, U + 0x64, 1) & 64) == 0) break;
                    }
                    scenario++;
                    out.flush();
                }
            } finally { e.dispose(); }
        }
    }
}
