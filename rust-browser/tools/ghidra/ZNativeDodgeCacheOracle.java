import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
import java.util.Arrays;

// Execute the GOG dodge, bucket gathering, RNG and replacement-line routines.
public class ZNativeDodgeCacheOracle extends ZNativeReactionsOracle {
    private static final long COUNT = 0x635b78, BUCKETS = 0x635c78, RESULT = 0x62d3a8;
    private static final long RNG = 0x622cdc, HARDWARE = 0x612f80;
    private static final int ORIGIN = 4 * 16 + 4;

    private void bucket(EmulatorHelper e, int index, int... slots) throws Exception {
        put(e, COUNT + index, 1, slots.length);
        for (int i = 0; i < slots.length; i++) {
            put(e, BUCKETS + index * 32 + i, 1, slots[i]);
        }
    }

    private void unit(EmulatorHelper e, int slot, int x, int y, int flags, int phase, int cls) throws Exception {
        long address = U + slot * 192;
        put(e, address + 2, 1, 60);
        put(e, address + 8, 4, cls == 1 ? HARDWARE : -1);
        put(e, address + 0x2c, 2, x);
        put(e, address + 0x2e, 2, y);
        put(e, address + 0x31, 1, flags);
        put(e, address + 0x35, 1, phase);
        put(e, address + 0x5e, 1, 1);
        put(e, address + 0x71, 1, 4);
        put(e, address + 0x7c, 1, cls);
        put(e, address + 0x8b, 1, 7);
    }

    @Override public void run() throws Exception {
        try (PrintWriter out = new PrintWriter(getScriptArgs()[0])) {
            EmulatorHelper e = new EmulatorHelper(currentProgram);
            try {
                e.setBreakpoint(toAddr(RET));
                e.setBreakpoint(toAddr(0x47326a));
                for (int scenario = 0; scenario < 32; scenario++) {
                    e.writeMemory(toAddr(U), new byte[192 * 8]);
                    e.writeMemory(toAddr(COUNT), new byte[256]);
                    e.writeMemory(toAddr(HARDWARE), new byte[44]);
                    e.writeMemory(toAddr(P), new byte[64]);
                    put(e, HARDWARE + 0x17, 1, 0x20);
                    put(e, 0x64498c, 4, 0x498000);
                    put(e, 0x622dc4, 1, (scenario >> 3) & 1);
                    for (int i = 0; i < 5; i++) put(e, RNG + i * 4, 4, 0xa5331991L + i * 0x123af911L);
                    unit(e, 0, 600, 520, (scenario & 1) == 0 ? 1 : 4, 0, 0);
                    unit(e, 1, 512, 512, 4, (scenario & 2) == 0 ? 1 : 7, 0);
                    unit(e, 2, 513, 511, 4, 58, 1);
                    unit(e, 3, 515, 515, 4, 0, 0);
                    unit(e, 4, (scenario & 4) == 0 ? 580 : 514, 512, 4, 1, 0);
                    unit(e, 5, 513, 511, 4, 0, 2);
                    unit(e, 6, 515, 515, 0, 1, 0);
                    bucket(e, ORIGIN, 1, 4, 5);
                    bucket(e, ORIGIN - 1, 2);
                    bucket(e, ORIGIN + 16, 6);
                    bucket(e, ORIGIN + 2, 3);
                    put(e, P, 2, (scenario & 16) == 0 ? 514 : 497);
                    put(e, P + 2, 2, 512);
                    put(e, P + 0x24, 1, 0);
                    put(e, P + 0x26, 2, 520);
                    put(e, P + 0x28, 2, 520);
                    put(e, S, 4, RET);
                    e.writeRegister("ESP", S);
                    e.writeRegister("ECX", P);
                    if (!e.run(toAddr(0x462fd0), null, monitor)) throw new Exception(e.getLastError());
                    // The CRT memcpy dispatch pointer is assigned by Windows startup.
                    // Transfer its bytes and return to the real bucket loop.
                    while (e.readRegister("EIP").longValue() == 0x47326a) {
                        long stack = e.readRegister("ESP").longValue();
                        long dest = read(e, stack + 4, 4);
                        long src = read(e, stack + 8, 4);
                        int size = (int)read(e, stack + 12, 4);
                        e.writeMemory(toAddr(dest), e.readMemory(toAddr(src), size));
                        e.writeRegister("EAX", dest);
                        e.writeRegister("ESP", stack + 4);
                        if (!e.run(toAddr(read(e, stack, 4)), null, monitor)) throw new Exception(e.getLastError());
                    }
                    if (e.readRegister("EIP").longValue() != RET) throw new Exception("Unexpected stop");

                    long[] rng = new long[5];
                    for (int i = 0; i < rng.length; i++) rng[i] = read(e, RNG + i * 4, 4);
                    int[] slots = new int[5];
                    for (int i = 0; i < slots.length; i++) slots[i] = (int)read(e, RESULT + i, 1);
                    out.printf("{\"scenario\":%d,\"slots\":%s,\"rng\":%s,\"shape\":%d,\"hardware\":%d,\"units\":[",
                        scenario, Arrays.toString(slots), Arrays.toString(rng), read(e, 0x64498c, 4), read(e, HARDWARE + 0x17, 1));
                    for (int slot = 1; slot <= 6; slot++) {
                        long u = U + slot * 192;
                        if (slot != 1) out.print(",");
                        out.print(Arrays.toString(new long[] {
                            read(e, u + 0x64, 1), read(e, u + 0x8c, 1), read(e, u + 0x35, 1),
                            read(e, u + 0x48, 2), read(e, u + 0x81, 1), read(e, u + 0x39, 1), read(e, u + 0x71, 1)
                        }));
                    }
                    out.println("]}");
                    out.flush();
                }
            } finally { e.dispose(); }
        }
    }
}
