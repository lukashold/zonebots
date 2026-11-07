// Executes original 43c1b0. Only the collision/shortcut helper 43bfb0 is
// fenced with declared outcomes. This verifies physical candidate shifts,
// not the collision decisions or a complete mission. No GOG files are changed.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;

public class ZCandidateShiftOracle extends GhidraScript {
    static final long BASE = 0x4d1480L, UNIT = 0x5e30c8L;
    static final long STACK = 0x7fff0000L, RETURN = 0x12345000L;
    long read(EmulatorHelper e, long address, int size) {
        long value = 0;
        byte[] bytes = e.readMemory(toAddr(address), size);
        for (int i = 0; i < size; i++) value |= (bytes[i] & 255L) << (i * 8);
        return value;
    }
    void put(EmulatorHelper e, long address, int size, long value) {
        e.writeMemoryValue(toAddr(address), size, value);
    }
    String hex(EmulatorHelper e) {
        StringBuilder out = new StringBuilder();
        for (byte value : e.readMemory(toAddr(BASE), 772))
            out.append(String.format("%02x", value & 255));
        return out.toString();
    }
    public void run() throws Exception {
        try (PrintWriter out = new PrintWriter(getScriptArgs()[0])) {
            for (int selected = 0; selected < 4; selected++)
            for (int count : new int[]{3, 8, 37, 49})
            for (int pattern = 0; pattern < 4; pattern++) {
                EmulatorHelper e = new EmulatorHelper(currentProgram);
                try {
                    byte[] raw = new byte[772];
                    for (int i = 0; i < raw.length; i++) raw[i] = (byte)(i * 23 + selected * 17 + 31);
                    e.writeMemory(toAddr(BASE), raw);
                    long candidate = BASE + 368 + selected * 100;
                    put(e, candidate, 1, 7);
                    put(e, candidate + 1, 1, -count);
                    for (int i = 0; i < count; i++) put(e, candidate + 2 + i * 2, 2, 1280 + i * 17);
                    put(e, UNIT + 0x2c, 2, 123);
                    put(e, UNIT + 0x2e, 2, 456);
                    String before = hex(e);
                    e.writeRegister("ESP", STACK);
                    e.writeRegister("ECX", UNIT);
                    e.writeRegister("EDX", candidate);
                    e.writeRegister("DF", 0);
                    put(e, STACK, 4, RETURN);
                    e.setBreakpoint(toAddr(RETURN));
                    e.setBreakpoint(toAddr(0x43bfb0));
                    if (!e.run(toAddr(0x43c1b0), null, monitor)) throw new Exception(e.getLastError());
                    int calls = 0;
                    StringBuilder removed = new StringBuilder();
                    while (e.readRegister("EIP").longValue() != RETURN) {
                        if (++calls > 49) throw new Exception("Shortcut call budget exceeded");
                        long sp = e.readRegister("ESP").longValue();
                        boolean clear = pattern == 1 || (pattern == 2 && calls % 2 == 0)
                            || (pattern == 3 && calls == count - 1);
                        if (clear) {
                            if (removed.length() != 0) removed.append(',');
                            removed.append((int)read(e, sp + 12, 4) + 1);
                        }
                        put(e, e.readRegister("ECX").longValue(), 4, UNIT);
                        put(e, read(e, sp + 4, 4), 2, 123);
                        put(e, read(e, sp + 8, 4), 2, 456);
                        e.writeRegister("EAX", clear ? 0 : 1);
                        long next = read(e, sp, 4);
                        e.writeRegister("ESP", sp + 4);
                        if (!e.run(toAddr(next), null, monitor)) throw new Exception(e.getLastError());
                    }
                    out.println("{\"selected\":" + selected + ",\"count\":" + count
                        + ",\"pattern\":" + pattern + ",\"calls\":" + calls
                        + ",\"removed\":[" + removed + "],\"before\":\"" + before
                        + "\",\"after\":\"" + hex(e) + "\",\"cursor\":" + read(e, BASE + 360, 4) + "}");
                    out.flush();
                } finally { e.dispose(); }
            }
        }
    }
}
