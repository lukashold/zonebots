import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
import java.util.Arrays;

// The parent runs the real 58ec0 removal and its original callees.
// This follows it with 462fd0 before the prepared UNIT list is rebuilt.
public class ZNativeRetiredDodgeOracle extends ZV44RemovedShooterHeaderOracle {
    private static final long PROJECTILE = 0x1100000, COUNT = 0x635b78, CELLS = 0x635c78;
    private int draws;

    private long[] fields(EmulatorHelper e) {
        return new long[] {
            get(e, U + 0x02, 1), get(e, U + 0x08, 4), get(e, U + 0x31, 1), get(e, U + 0x35, 1),
            get(e, U + 0x64, 1), get(e, U + 0x69, 1), get(e, U + 0x71, 1), get(e, U + 0x8c, 1),
            get(e, U + 0x48, 2), get(e, U + 0x81, 1), get(e, U + 0x39, 1), get(e, 0x62dc1f, 1),
            get(e, 0x622cdc, 4), get(e, 0x622ce0, 4), get(e, 0x622ce4, 4),
            get(e, 0x622ce8, 4), get(e, 0x622cec, 4)
        };
    }

    private void projectile(EmulatorHelper e) throws Exception {
        put(e, S, 4, RET);
        e.writeRegister("ESP", S);
        e.writeRegister("ECX", PROJECTILE);
        if (!e.run(toAddr(0x462fd0), null, monitor)) throw new Exception(e.getLastError());
        for (int count = 0; e.readRegister("EIP").longValue() != RET; count++) {
            if (count > 50) throw new Exception("Unexpected dodge call sequence");
            long pc = e.readRegister("EIP").longValue();
            if (pc == 0x47326a) {
                long sp = e.readRegister("ESP").longValue();
                long dest = get(e, sp + 4, 4), src = get(e, sp + 8, 4);
                e.writeMemory(toAddr(dest), e.readMemory(toAddr(src), (int)get(e, sp + 12, 4)));
                e.writeRegister("EAX", dest);
                e.writeRegister("ESP", sp + 4);
                if (!e.run(toAddr(get(e, sp, 4)), null, monitor)) throw new Exception(e.getLastError());
            } else {
                if (pc == 0x460910) draws++;
                next(e);
            }
        }
    }

    @Override void row(PrintWriter out, EmulatorHelper e, int scenario, String stage, String trace, int rng) throws Exception {
        if (stage.equals("before")) {
            put(e, U + 2, 1, (scenario & 4) == 0 ? 60 : 0);
        } else if (stage.equals("after")) {
            int x = (short)get(e, U + 0x2c, 2), y = (short)get(e, U + 0x2e, 2);
            int bucket = 16 * (y >> 7) + (x >> 7);
            e.writeMemory(toAddr(COUNT), new byte[256]);
            if (bucket < 0 || bucket >= 256) return;
            put(e, COUNT + bucket, 1, 1);
            put(e, CELLS + bucket * 32, 1, 0);
            e.writeMemory(toAddr(PROJECTILE), new byte[64]);
            put(e, PROJECTILE, 2, x);
            put(e, PROJECTILE + 2, 2, y);
            put(e, PROJECTILE + 0x24, 1, 7);
            put(e, PROJECTILE + 0x26, 2, x + 8);
            put(e, PROJECTILE + 0x28, 2, y + 8);
            put(e, U + 7 * 192 + 0x31, 1, (scenario & 8) == 0 ? 0 : 1);
            e.setBreakpoint(toAddr(0x47326a));
            long[] before = fields(e);
            draws = 0;
            projectile(e);
            long[] after = fields(e);
            int first = draws;
            projectile(e);
            out.printf("{\"scenario\":%d,\"before\":%s,\"after\":%s,\"repeat\":%s,\"draws\":%d,\"repeatDraws\":%d}%n",
                scenario, Arrays.toString(before), Arrays.toString(after), Arrays.toString(fields(e)), first, draws - first);
            out.flush();
        }
    }
}
