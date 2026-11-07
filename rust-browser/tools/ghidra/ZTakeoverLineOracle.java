// Executes complete50be0, including581f0, vehicle constructors and52500.
// The later50cd0 group reconstruction/UI suffix is outside this oracle.
// Input equipment attributes are declared synthetic controls; ROBOTS.DAT
// only supplies the local original profile tables. No gameplay calls fenced.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
import java.nio.file.Files;
import java.nio.file.Path;
public class ZTakeoverLineOracle extends GhidraScript {
 static final long U=0x5e30c8L,G=0x60ec00L,W=0x59e6c2L,P=0x5eee28L,S=0x7fff0000L,RET=0x12345000L,Q=0x1900000L;
 long read(EmulatorHelper e,long p,int n){long r=0;byte[] b=e.readMemory(toAddr(p),n);for(int i=0;i<n;i++)r|=(b[i]&255L)<<(i*8);return r;}
 void put(EmulatorHelper e,long p,int n,long v){e.writeMemoryValue(toAddr(p),n,v);}
 String hex(EmulatorHelper e,long p,int n){StringBuilder s=new StringBuilder();for(byte b:e.readMemory(toAddr(p),n))s.append(String.format("%02x",b&255));return s.toString();}
 void call(EmulatorHelper e,long f,long cx,long dx,long a)throws Exception{e.writeRegister("ESP",S);put(e,S,4,RET);put(e,S+4,4,a);e.writeRegister("ECX",cx);e.writeRegister("EDX",dx);if(!e.run(toAddr(f),null,monitor)||e.readRegister("EIP").longValue()!=RET)throw new Exception("Native call "+Long.toHexString(f)+": "+e.getLastError());}
 public void run()throws Exception{
  byte[] robots=Files.readAllBytes(Path.of("C:/GOG Games/Z/robots.dat"));
  int[][] delta={{24,0},{24,12},{0,24},{-24,12},{-24,0},{-24,-12},{0,-24},{24,-12}};
  try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
   for(int definition:new int[]{8,24,26,29,31,33,35,37})for(int direction=0;direction<8;direction++){
    EmulatorHelper e=new EmulatorHelper(currentProgram);try{
     e.setBreakpoint(toAddr(RET));for(String r:new String[]{"EBP","EBX","ESI","EDI","DF"})e.writeRegister(r,0);
     put(e,0x622dc4,1,0);put(e,0x622c7a,2,0);put(e,0x625eae,1,0);call(e,0x433b30,0,0,0);call(e,0x44b740,0,0,0);
     e.writeMemory(toAddr(0x615498),robots);e.writeMemory(toAddr(U),new byte[240*192]);e.writeMemory(toAddr(G),new byte[240*72]);e.writeMemory(toAddr(W),new byte[240*44]);e.writeMemory(toAddr(P),new byte[64]);e.writeMemory(toAddr(Q),new byte[256]);
     for(int i=0;i<240;i++)put(e,W+i*44,2,-1);
     put(e,Q+0x1c,1,2);put(e,0x63238c,4,Q);put(e,0x5849d3,1,0);put(e,0x584963,1,0);put(e,0x584962,1,0);put(e,0x57c952,1,0);put(e,0x622df9,1,0);
     call(e,0x41b210,320,320,Q+128);if(read(e,Q+128,4)!=U)throw new Exception("Unexpected UNIT allocation");
     put(e,U+0x31,1,4);put(e,U+0x5e,1,-1);put(e,U+0x5f,1,0);put(e,G+18,1,1);put(e,G+19,1,0);put(e,0x632194,4,G);
     int[] d=delta[direction];put(e,P,2,328+d[0]);put(e,P+2,2,328+d[1]);put(e,P+4,4,0);put(e,U+20,4,P);call(e,0x4525a0,U,320+d[0],320+d[1]);
     put(e,U+0x30,1,definition);long input=Q+64;put(e,input+14,1,20);put(e,input+15,1,8);put(e,input+16,1,30);put(e,input+17,1,10);put(e,input+24,1,80);put(e,input+25,1,15);put(e,input+40,1,8);
     String before=hex(e,U,192);call(e,0x450be0,U,input,0);
     out.println("{\"definition\":"+definition+",\"direction\":"+direction+",\"before\":\""+before+"\",\"after\":\""+hex(e,U,192)+"\",\"hardware\":\""+hex(e,W,44)+"\",\"node\":\""+hex(e,P,8)+"\"}");out.flush();
    }finally{e.dispose();}
   }
  }
 }
}
