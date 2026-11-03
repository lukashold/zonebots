// Numeric execution of native idle initialization and normal state1 callbacks.
// World scans/follower routing are intercepted; local RNG/counters are native.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeIdleOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x1000000L,G=U+0x1000;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 void invoke(EmulatorHelper e,long a,boolean fast)throws Exception{e.writeRegister("ESP",S);put(e,S,4,RET);put(e,S+4,4,U);e.writeRegister("ECX",U);if(!e.run(toAddr(a),null,monitor))throw new Exception(e.getLastError());}
 void emit(PrintWriter out,EmulatorHelper e,int aim,int chance,int direction,int multi,int tick,int search,int wx,int wy)throws Exception{
  out.println("{\"aim\":"+aim+",\"chance\":"+chance+",\"initialDirection\":"+direction+",\"multi\":"+multi+",\"tick\":"+tick+",\"state\":"+read(e,U+0x35,1)+",\"frame\":"+read(e,U+0x32,2)+",\"direction\":"+read(e,U+0x39,1)+",\"timer\":"+(byte)read(e,U+0x3c,1)+",\"looks\":"+(byte)read(e,U+0x3d,1)+",\"desired\":"+read(e,U+0x40,1)+",\"step\":"+(byte)read(e,U+0x41,1)+",\"scanTimer\":"+read(e,U+0x7d,1)+",\"flags69\":"+read(e,U+0x69,1)+",\"search\":"+search+",\"wx\":"+wx+",\"wy\":"+wy+",\"rng\":["+read(e,0x622cdc,4)+","+read(e,0x622ce0,4)+","+read(e,0x622ce4,4)+","+read(e,0x622ce8,4)+","+read(e,0x622cec,4)+"]}");
 }
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int aim:new int[]{0,40,60,120})for(int chance:new int[]{0,60,127})for(int direction:new int[]{0,3,7})for(int multi=0;multi<2;multi++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{
    e.writeMemory(toAddr(U),new byte[0x2000]);put(e,0x632194,4,G);put(e,G+0x3b,1,255);put(e,U+0x5e,1,1);put(e,U+0x30,1,4);put(e,U+0x2c,2,500);put(e,U+0x2e,2,600);put(e,U+2,1,aim);put(e,U+0x3e,1,chance);put(e,U+0x39,1,direction);put(e,U+0x69,1,128);put(e,0x622dc4,1,multi);for(int i=0;i<5;i++)put(e,0x622cdc+i*4,4,i+1);
    for(long a:new long[]{RET,0x44d3a0,0x44d140,0x458500,0x44d570,0x459e40,0x4525a0})e.setBreakpoint(toAddr(a));
    invoke(e,0x44cac0,true);if(e.readRegister("EIP").longValue()!=RET)throw new Exception("Unexpected idle initializer stop");emit(out,e,aim,chance,direction,multi,0,0,0,0);
    for(int tick=1;tick<=240;tick++){
     put(e,0x622dc0,1,tick&1);invoke(e,0x44d920,false);int search=0,wx=0,wy=0;
     while(e.readRegister("EIP").longValue()!=RET){long a=e.readRegister("EIP").longValue(),s=e.readRegister("ESP").longValue();
      if(a==0x44d3a0||a==0x44d140){put(e,read(e,s+4,4),1,255);put(e,read(e,s+8,4),1,255);search|=a==0x44d3a0?1:2;}
      else if(a==0x458500){e.writeRegister("EAX",0xff00);search|=4;}
      else if(a==0x44d570)search|=8;
      else if(a==0x459e40){}
      else if(a==0x4525a0){search|=16;wx=(short)e.readRegister("EDX").intValue();wy=(short)read(e,s+4,4);break;}
      else throw new Exception("Unexpected idle callback stop"+Long.toHexString(a));resume(e);
     }
     emit(out,e,aim,chance,direction,multi,tick,search,wx,wy);if((search&16)!=0)break;
    }
   }finally{e.dispose();}
  }
 }}
}
