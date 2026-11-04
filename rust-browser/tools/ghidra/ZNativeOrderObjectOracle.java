// Read-only original45f070/43ea80 projection. Run against a local licensed Z.exe.
// Classified C3=1 inputs deliberately exclude the selection/UI/network caller.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
import java.util.Arrays;

public class ZNativeOrderObjectOracle extends GhidraScript {
    static final long OBJECT=0x612f80L,TEMPLATES=0x1900000L,ACTOR=0x5e30c8L+5*192,
        STACK=0x7fff0000L,RETURN=0x12345000L;
    void put(EmulatorHelper e,long a,int n,long v){e.writeMemoryValue(toAddr(a),n,v);}
    long get(EmulatorHelper e,long a,int n){byte[] b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
    public void run()throws Exception {
        try(PrintWriter out=new PrintWriter(getScriptArgs()[0],"UTF-8")) {
            int[] opcodes={2,5,4,8,9,11,12,13},offsets={-1,0,7,8,15,16,23,24,32767,-32768};
            for(int seed=0;seed<512;seed++) {
                EmulatorHelper e=new EmulatorHelper(currentProgram);
                try {
                    e.writeMemory(toAddr(ACTOR),new byte[192]);
                    e.writeMemory(toAddr(OBJECT),new byte[200*44]);
                    for(int i=0;i<200;i++) {
                        put(e,OBJECT+i*44L,2,-1);put(e,OBJECT+i*44L+4,4,TEMPLATES+i*24L);
                    }
                    for(int i=0;i<4;i++) {
                        put(e,OBJECT+i*44L,2,new int[]{300,301,-2,32760}[i]);
                        put(e,OBJECT+i*44L+2,2,new int[]{100,101,-2,32760}[i]);
                        put(e,TEMPLATES+i*24L+18,1,i==0?seed*13:16+(seed+i)%4);
                    }
                    int op=opcodes[seed%8];short qx=(short)(300+offsets[seed/8%10]),qy=(short)(100+offsets[seed/80%6]);
                    put(e,ACTOR+0xb3,1,99);put(e,0x4d18c8,4,0x12340000L+seed);
                    put(e,0x4d02c3,1,1);put(e,0x4d02c6,1,0);put(e,0x622dc4,1,0);
                    put(e,0x56ed47,1,op);put(e,0x62dc04,4,ACTOR);
                    put(e,0x62dc00,2,op==5?200:qx);put(e,0x62dc3c,2,op==5?qx:qy);put(e,0x62dc48,2,qy);
                    for(int i=0;i<5;i++)put(e,0x622cdc+i*4,4,0x173510a7L+seed*7919L+i*0x9e3779b9L);
                    byte[] rng=e.readMemory(toAddr(0x622cdc),20);
                    for(long p:new long[]{RETURN,0x43ea80L,0x460910L,0x47326aL})e.setBreakpoint(toAddr(p));
                    put(e,STACK,4,RETURN);e.writeRegister("ESP",STACK);
                    if(!e.run(toAddr(0x45f070),null,monitor))throw new Exception(e.getLastError());
                    int queries=0;
                    while(e.readRegister("EIP").longValue()!=RETURN) {
                        long pc=e.readRegister("EIP").longValue();
                        if(pc!=0x43ea80L)throw new Exception("Unexpected RNG/copy leaf "+Long.toHexString(pc));
                        queries++;
                        if(!e.step(monitor)||!e.run(monitor))throw new Exception(e.getLastError());
                    }
                    if(!Arrays.equals(rng,e.readMemory(toAddr(0x622cdc),20)))throw new Exception("RNG changed");
                    out.println("{\"scenario\":"+seed+",\"opcode\":"+op+",\"b3\":"+get(e,ACTOR+0xb3,1)+",\"pointer\":"+get(e,0x4d18c8,4)+",\"rngCalls\":0,\"copyCalls\":0,\"queryCalls\":"+queries+"}");out.flush();
                }finally{e.dispose();}
            }
        }
    }
}
