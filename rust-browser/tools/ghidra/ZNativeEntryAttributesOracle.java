//Actual43eee0 equipment projection followed by581f0 first-free takeover.
//The local GOG hardware table is loaded privately; output contains checksums
//and typed numeric gameplay fields only, never the original table bytes.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
import java.nio.file.*;
public class ZNativeEntryAttributesOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,O=0x612f80L,W=0x59e6c2L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 long hash(byte[]b){long h=2166136261L;for(byte n:b)h=((h^(n&255L))*16777619L)&0xffffffffL;return h;}
 void call(EmulatorHelper e,long f,long cx,long dx)throws Exception{put(e,S,4,RET);e.writeRegister("ESP",S);e.writeRegister("ECX",cx);e.writeRegister("EDX",dx);e.writeRegister("EIP",f);e.getEmulator().setHalt(false);for(int n=0;n<100000;n++){long pc=e.readRegister("EIP").longValue();if(pc==RET)return;if(!e.step(monitor))throw new Exception(e.getLastError());}throw new Exception("CPU budget at "+e.readRegister("EIP").toString(16));}
 public void run()throws Exception{String base=System.getenv("Z_GAME_DIR");if(base==null)base="C:/GOG Games/Z";byte[]robots=Files.readAllBytes(Paths.get(base,"robots.dat"));try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int equipment=0;equipment<20;equipment++)for(int scenario=0;scenario<16;scenario++){
  EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.writeMemory(toAddr(0x61aa88),java.util.Arrays.copyOfRange(robots,0x55f0,0x55f0+19*42));byte[]object=new byte[44];for(int i=0;i<44;i++)object[i]=(byte)(scenario*7+i*13);e.writeMemory(toAddr(O),object);e.writeMemory(toAddr(U),new byte[192]);int free=scenario%5;for(int slot=0;slot<240;slot++){byte[]r=new byte[44];for(int i=0;i<44;i++)r[i]=(byte)((slot*44+i)*11+scenario*3);e.writeMemory(toAddr(W+slot*44),r);put(e,W+slot*44,2,slot<free?0:-1);}int eq=equipment==19?255:equipment;call(e,0x43eee0,O,eq);long objectHash=hash(e.readMemory(toAddr(O),44));put(e,U+0x78,1,scenario*7);put(e,U+0x79,1,scenario*11);put(e,U+0x7b,1,127);call(e,0x4581f0,U,O);out.println(eq+","+scenario+","+free+","+objectHash+","+hash(e.readMemory(toAddr(W+free*44),44))+","+read(e,U+0x78,1)+","+read(e,U+0x79,1)+","+read(e,U+4,1)+","+read(e,U+0x7b,1)+","+((read(e,U+8,4)-W)/44));out.flush();}finally{e.dispose();}
 }}}
}
