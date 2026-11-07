// Executes actual 457590/4d570, their finite paths, line setup and movement.
// No leaf return values are substituted. Requires the owner's local SINCOS.DAT.
// Arguments: output.jsonl start|motion|blocked|direct
// Scans are disabled; movement cases stop at the first handoff to another phase.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

public class ZCoupledFollowerOracle extends GhidraScript {
    static final long U=0x5e30c8L, R=0x60ec00L, P=0x5eee28L;
    static final long S=0x7fff0000L, T=0x71000000L, M=0x72000000L, RET=0x12345000L;

    long get(EmulatorHelper e,long address,int size) {
        byte[] bytes=e.readMemory(toAddr(address),size); long result=0;
        for(int i=0;i<size;i++) result|=(bytes[i]&255L)<<(i*8);
        return result;
    }
    void put(EmulatorHelper e,long address,int size,long value)throws Exception {
        e.writeMemoryValue(toAddr(address),size,value);
    }
    void call(EmulatorHelper e,long entry,long ecx,long... args)throws Exception {
        e.writeMemory(toAddr(S-1024),new byte[2048]);
        for(String reg:new String[]{"EBP","EBX","ESI","EDI","EDX"}) e.writeRegister(reg,0);
        e.writeRegister("ESP",S); e.writeRegister("ECX",ecx); put(e,S,4,RET);
        for(int i=0;i<args.length;i++) put(e,S+4+i*4,4,args[i]);
        if(!e.run(toAddr(entry),null,monitor)) throw new Exception(e.getLastError());
        if(e.readRegister("EIP").longValue()!=RET) throw new Exception("Unexpected return");
    }
    void setup(EmulatorHelper e,int scenario,boolean direct,boolean blocked,byte[] trig)throws Exception {
        e.setBreakpoint(toAddr(RET));
        byte[] terrain=new byte[16384];
        if(blocked) Arrays.fill(terrain,(byte)19);
        e.writeMemory(toAddr(0x58aaaa),terrain);
        put(e,0x622dc4,1,0); put(e,0x622c7a,2,0); put(e,0x625eae,1,0);
        call(e,0x433b30,0);
        e.writeMemory(toAddr(U),new byte[240*192]);
        e.writeMemory(toAddr(R),new byte[240*72]);
        e.writeMemory(toAddr(0x62dc08),new byte[44]);
        byte[] free=new byte[14400*8];
        for(int i=0;i<14400;i++) Arrays.fill(free,i*8+4,i*8+8,(byte)255);
        e.writeMemory(toAddr(P),free);
        long u=U+192;
        int x=304+new int[]{0,4,-4,16,-16,48,-48,64}[scenario%8];
        int y=335+new int[]{0,7,-7,40}[scenario/8%4];
        boolean hardware=direct&&scenario>=256;
        put(e,U+0x2c,2,320); put(e,U+0x2e,2,320);
        put(e,u+0x2c,2,x); put(e,u+0x2e,2,y);
        put(e,u+8,4,-1); put(e,u+0x30,1,hardware?34:4); put(e,u+0x34,1,255);
        put(e,u+0x7c,1,hardware?1:0); put(e,u+0x31,1,4); put(e,u+0x35,1,hardware?9:7);
        put(e,u+0x32,2,scenario*13%300); put(e,u+0x3c,1,scenario%5);
        put(e,u+0x39,1,scenario%8); put(e,u+0x48,2,0);
        put(e,u+0x4a,2,123); put(e,u+0x4c,2,456); put(e,u+0x4e,2,789);
        put(e,u+0x5e,1,1); put(e,u+0x8b,1,scenario%13); put(e,u+0xa2,2,0x1234);
        put(e,R+9,1,1); put(e,R+17,1,6); put(e,R+19,1,0); put(e,R+20,1,1); put(e,R+59,1,255);
        put(e,0x632194,4,R); put(e,0x622dc0,1,0); put(e,0x4d029b,1,0);
        put(e,0x586a80,4,T);
        for(int i=0;i<640;i++) put(e,T+i*2,2,(short)(((trig[i*2]&255)<<8)|(trig[i*2+1]&255))>>5);
        // Standard first follower at angle0; the original table uses cosine1023.
        put(e,0x49a050,4,M); put(e,M,1,-16); put(e,M+1,1,16);
        call(e,0x41b950,u);
        if(scenario%256>=128) {
            put(e,u+0x14,4,P); put(e,P,2,x+12); put(e,P+2,2,y+10); put(e,P+4,4,0);
        }
    }
    String fields(EmulatorHelper e)throws Exception {
        long u=U+192;
        int[] offsets={0x2c,0x2e,0x32,0x35,0x3c,0x39,0x31,0x48,0x4a,0x4c,0x4e,0x81,0x8c,0xa2};
        StringBuilder values=new StringBuilder();
        for(int offset:offsets) {
            int size=(offset==0x2c||offset==0x2e||offset==0x32||offset==0xa2||(offset>=0x48&&offset<=0x4e))?2:1;
            long value=get(e,u+offset,size);
            if(size==2) value=(short)value;
            if(offset==0x3c) value=(byte)value;
            if(values.length()!=0) values.append(',');
            values.append(value);
        }
        long head=get(e,u+0x14,4);
        values.append(',').append(head).append(',').append(get(e,0x62dc08+23,1));
        if(head!=0) values.append(',').append((short)get(e,head,2)).append(',').append((short)get(e,head+2,2)).append(',').append(get(e,head+4,4));
        return values.toString();
    }
    public void run()throws Exception {
        String[] args=getScriptArgs();
        if(args.length!=2||!Arrays.asList("start","motion","blocked","direct").contains(args[1]))
            throw new IllegalArgumentException("output.jsonl start|motion|blocked|direct");
        String mode=args[1]; boolean direct=mode.equals("direct"), blocked=mode.equals("blocked");
        boolean moving=blocked||mode.equals("motion");
        byte[] trig=Files.readAllBytes(Path.of(System.getenv().getOrDefault("Z_GAME_DIR","C:/GOG Games/Z"),"SINCOS.DAT"));
        if(trig.length<1280) throw new IllegalArgumentException("Incomplete local trigonometry table");
        try(PrintWriter out=new PrintWriter(args[0])) {
            for(int scenario=0;scenario<(direct?512:256);scenario++) {
                monitor.checkCancelled(); EmulatorHelper e=new EmulatorHelper(currentProgram);
                try {
                    setup(e,scenario,direct,blocked,trig);
                    for(int tick=0;tick<=(moving?12:0);tick++) {
                        if(tick>0&&get(e,U+192+0x35,1)!=7) break;
                        put(e,U+192+0x62,1,tick==0?0:scenario%4+1);
                        if(direct) call(e,0x44d570,U+192); else call(e,0x457590,0,U+192);
                        out.printf("{\"scenario\":%d,\"tick\":%d,\"values\":[%s]}%n",scenario,tick,fields(e));
                        out.flush();
                    }
                } finally { e.dispose(); }
            }
        }
    }
}
