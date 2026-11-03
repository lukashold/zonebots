import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
public class ZNativeDirectGeometryOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,OUT=0x1100000L;
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 public void run()throws Exception{try(PrintWriter o=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{
  e.setBreakpoint(toAddr(RET));for(int s=0;s<512;s++){
   for(int i=0;i<8;i++){long p=U+i*0xc0;put(e,p+0x2c,2,500+(s*13+i*7)%81-40);put(e,p+0x2e,2,500+(s*17+i*11)%81-40);put(e,p+0x7c,1,(s+i)%8);put(e,0x635c78+32*51+i,1,i);}
   put(e,0x637d14,2,484);put(e,0x637ccc,2,484);put(e,0x635b64,2,516);put(e,0x635b68,2,516);e.writeRegister("ESP",S);put(e,S,4,RET);put(e,S+4,4,8);e.writeRegister("ECX",OUT);e.writeRegister("EDX",51);if(!e.run(toAddr(0x4565a0),null,monitor))throw new Exception(e.getLastError());int count=(int)(e.readRegister("EAX").longValue()-OUT),mask=0;for(int i=0;i<count;i++)mask|=1<<(int)read(e,OUT+i,1);o.println("{\"scenario\":"+s+",\"mask\":"+mask+"}");
  }
 }finally{e.dispose();}}}
}
