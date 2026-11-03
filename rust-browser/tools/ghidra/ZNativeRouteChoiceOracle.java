// Executes original0043ba60 final road-vs-direct comparison. Native price
// calls receive controlled values; allocation is an explicit boundary.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeRouteChoiceOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x1000000L,P=U+0x1000;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int road:new int[]{0,5,10,0xfffffff})for(int direct:new int[]{0,5,10})for(int special=0;special<3;special++)for(int level:new int[]{0,5,6})for(int multi=0;multi<2;multi++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{
    e.writeMemory(toAddr(U),new byte[0x2000]);put(e,P,2,100);put(e,P+2,2,200);put(e,P+4,4,P+8);put(e,P+8,2,300);put(e,P+10,2,400);put(e,0x4d15ec,4,P);put(e,0x4d15e0,4,U);put(e,0x622c7a,2,multi==0?level:99);put(e,0x622cbc,2,multi==1?level:99);put(e,0x622dc4,1,multi);
    for(long a:new long[]{RET,0x45cf90,0x43b640})e.setBreakpoint(toAddr(a));e.writeRegister("ESP",S);put(e,S,4,RET);if(!e.run(toAddr(0x43ba60),null,monitor))throw new Exception(e.getLastError());int calls=0,chosen=0;
    while(e.readRegister("EIP").longValue()!=RET){long a=e.readRegister("EIP").longValue();if(a==0x45cf90){e.writeRegister("EAX",calls++==0?road:direct);put(e,0x4d2be4,2,special);resume(e);}else if(a==0x43b640){chosen=1;break;}else throw new Exception("Unexpected route comparison stop");}
    out.println("{\"road\":"+road+",\"direct\":"+direct+",\"special\":"+special+",\"level\":"+level+",\"multi\":"+multi+",\"chosenDirect\":"+chosen+"}");
   }finally{e.dispose();}
  }
 }}
}
