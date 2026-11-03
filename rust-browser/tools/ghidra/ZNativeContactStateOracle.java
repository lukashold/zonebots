// Numeric knockdown and stand-up traces; no original code or art is exported.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeContactStateOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x1000000L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void invoke(EmulatorHelper e,long a)throws Exception{e.writeRegister("ESP",S);put(e,S,4,RET);put(e,S+4,4,U);e.writeRegister("ECX",U);if(!e.run(toAddr(a),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int facing=0;facing<8;facing++)for(int hp:new int[]{11,20,100}){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{e.writeMemory(toAddr(U),new byte[0xc0]);put(e,U+0x2c,2,500);put(e,U+0x2e,2,500);put(e,U+0x39,1,facing);put(e,U+0x3a,1,hp);put(e,U+0x35,1,1);
    for(long a:new long[]{RET,0x44cac0})e.setBreakpoint(toAddr(a));invoke(e,0x45b9c0);
    for(int tick=1;tick<=28;tick++){
     put(e,0x622dc0,1,tick&1);put(e,0x586976,1,1);put(e,0x4d1924,2,tick<4?500:524);put(e,0x4d1928,2,500);
     long state=read(e,U+0x35,1);invoke(e,state==23||state==46?0x45a6b0:0x45a740);
     boolean release=e.readRegister("EIP").longValue()==0x44cac0;
     out.println("{\"facing\":"+facing+",\"hpBefore\":"+hp+",\"tick\":"+tick+",\"state\":"+read(e,U+0x35,1)+",\"frame\":"+read(e,U+0x32,2)+",\"height\":"+(byte)read(e,U+0x76,1)+",\"hp\":"+(byte)read(e,U+0x3a,1)+",\"release\":"+release+"}");if(release)break;
    }
   }finally{e.dispose();}
  }
 }}
}
