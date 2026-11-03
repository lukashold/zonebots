// Full parent-particle traces through expiry6/7; child allocation/contact calls
// remain recorded boundaries, independently tested by the selector oracles.
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
import java.nio.file.Files;
import java.nio.file.Path;
public class ZNativeWorldEffectTraceOracle extends ZNativeWorldEffectExpiryOracle{
 @Override public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{
  for(long a:new long[]{0x460910,0x420200,0x41f2d0,0x402160,0x46e660,0x46e6d0,0x4666c0,0x466ac0,0x40bb50})e.setBreakpoint(toAddr(a));byte[]raw=Files.readAllBytes(Path.of("C:/GOG Games/Z/SINCOS.DAT"));for(int i=0;i<640;i++){int v=(short)(((raw[i*2]&255)<<8)|(raw[i*2+1]&255));put(e,TRIG+i*2,2,v>>5);}put(e,0x586a80,4,TRIG);put(e,0x477054,2,471);put(e,0x47705c,2,-256);put(e,0x477058,2,425);put(e,0x477060,2,1);put(e,0x622d50,2,4096);put(e,0x622d52,2,4096);put(e,0x6323a0,1,1);int s=0;
  for(int kind=0;kind<2;kind++)for(seed=0;seed<32;seed++){
   initializeSpecial(e,kind,seed);put(e,0x622cb8,2,seed%3*12);put(e,0x622cba,2,seed%5*8);put(e,0x625e3c,2,0);put(e,0x622be4,2,0);put(e,0x498c24,2,2048);put(e,0x498c20,2,2048);put(e,0x4a6e72,1,0);put(e,0x4a6f42,1,0);put(e,0x4a6e6f,1,0);put(e,0x4a6520,2,0);put(e,0x4a6530,2,0);byte[]plane=new byte[16384];int code=new int[]{0,9,29,42,38,40,39,7}[seed%8];java.util.Arrays.fill(plane,(byte)code);e.writeMemory(toAddr(0x58aaaa),plane);
   for(int tick=0;tick<128;tick++){
    put(e,0x4d18ee,1,0);eventHash=0x811c9dc5;eventCount=0;dx=-999;dy=-999;sprite=-1;height=0;angle=0;transformed=0;if(read(e,O,2)!=65536-1000)run(e,0x4028a0,O,0);
    out.println("{\"kind\":\"expiryTrace\",\"scenario\":"+s+++",\"hash\":"+hash(e.readMemory(toAddr(O),44))+",\"calls\":"+calls+",\"events\":"+eventCount+",\"eventHash\":"+(eventHash&0xffffffffL)+",\"quiet\":"+read(e,0x4a6e6f,1)+",\"soundGate\":"+read(e,0x4d18ee,1)+",\"sprite\":"+sprite+",\"x\":"+dx+",\"y\":"+dy+",\"height\":"+height+",\"angle\":"+angle+",\"transformed\":"+transformed+"}");
   }
  }
 }finally{e.dispose();}}}
}
