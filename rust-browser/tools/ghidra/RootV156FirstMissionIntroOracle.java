// Entry tail after the original loader: mission number, fresh-start flag,
// and native media-open/play leaves. Test the original executable's branch.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
public class RootV156FirstMissionIntroOracle extends GhidraScript {
 static final long S=0x7fff0000L,HALT=0x12345100L;
 long read(EmulatorHelper e,long a,int n){byte[] b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 String stringAt(EmulatorHelper e,long a){StringBuilder b=new StringBuilder();for(int i=0;i<80;i++){int n=(int)read(e,a+i,1);if(n==0)return b.toString();if(n<32||n>126)throw new IllegalArgumentException("string");b.append((char)n);}throw new IllegalArgumentException("length");}
 public void run()throws Exception{
  EmulatorHelper e=new EmulatorHelper(currentProgram);
  try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
   for(long a:new long[]{0x428f76,0x428f8a,HALT,0x431110,0x431820})e.setBreakpoint(toAddr(a));
   put(e,0x474220,4,HALT);
   for(int mission:new int[]{1,2,20})for(int fresh=0;fresh<2;fresh++)for(int media=0;media<3;media++){
    for(String r:new String[]{"EAX","ECX","EDX","EDI","EBP","DF"})e.writeRegister(r,0);
    e.writeRegister("EBX",fresh);e.writeRegister("ESI",2);e.writeRegister("ESP",S);
    put(e,0x622c7a,2,mission);put(e,0x622e20,1,0);put(e,0x4d02f0,4,123);put(e,0x4d02f4,2,99);put(e,0x64d7c4,2,0);
    if(!e.run(toAddr(0x428f1a),null,monitor))throw new Exception(e.getLastError());
    String movie="";int played=0,halt=0;
    for(int count=0;;count++){
     long pc=e.readRegister("EIP").longValue();if(pc==0x428f76||pc==0x428f8a)break;
     if(count>8)throw new Exception("call limit");
     long sp=e.readRegister("ESP").longValue();
     if(pc==HALT)halt++;
     else if(pc==0x431110){movie=stringAt(e,e.readRegister("ECX").longValue());e.writeRegister("EAX",media==0?0:1);}
     else if(pc==0x431820){played++;e.writeRegister("EAX",media==2?1:0);}
     else throw new Exception(Long.toHexString(pc));
     e.writeRegister("EIP",read(e,sp,4));e.writeRegister("ESP",sp+4);
     if(!e.run(monitor))throw new Exception(e.getLastError());
    }
    out.printf("{\"mission\":%d,\"fresh\":%d,\"media\":%d,\"movie\":\"%s\",\"played\":%d,\"halt\":%d,\"next\":%d,\"movieMode\":%d,\"startFlag\":%d}%n",mission,fresh,media,movie,played,halt,read(e,0x64d7c4,2),read(e,0x4d02f4,2),read(e,0x622e20,1));
   }
  }finally{e.dispose();}
 }
}
