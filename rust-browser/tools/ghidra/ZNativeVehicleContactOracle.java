// Native vehicle/robot inner and outer contact rectangles0045c210.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeVehicleContactOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,V=U+0xc0;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{
  try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
   for(int bridge=0;bridge<2;bridge++)for(int dx:new int[]{-24,-23,-19,-18,-9,-8,0,16,17,26,27,31,32})for(int dy:new int[]{-24,-23,-19,-18,-9,-8,0,16,17,22,23,27,28}){
    EmulatorHelper e=new EmulatorHelper(currentProgram);
    try{
     e.writeMemory(toAddr(U),new byte[0x180]);put(e,U+0x2c,2,500);put(e,U+0x2e,2,500);put(e,U+0x31,1,4);put(e,0x58aaaa,1,bridge!=0?7:0);
     put(e,V+0x2c,2,500+dx);put(e,V+0x2e,2,500+dy);put(e,V+0x31,1,5);put(e,V+0x3a,1,30);
     for(long a:new long[]{RET,0x44cd60,0x4525a0,0x458ec0})e.setBreakpoint(toAddr(a));
     e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",U);
     if(!e.run(toAddr(0x45c210),null,monitor))throw new Exception(e.getLastError());int action=0,x=0,y=0;
     while(e.readRegister("EIP").longValue()!=RET){long a=e.readRegister("EIP").longValue(),s=e.readRegister("ESP").longValue();if(a==0x44cd60){put(e,0x62d3a8,1,1);put(e,0x62d3a9,1,255);}else if(a==0x4525a0){action=3;x=e.readRegister("EDX").intValue();y=(int)read(e,s+4,4);}else if(a==0x458ec0)action=1;else throw new Exception("Unexpected contact stop");resume(e);}
     if(read(e,V+0x35,1)==23)action=2;
     out.println("{\"bridge\":"+bridge+",\"dx\":"+dx+",\"dy\":"+dy+",\"action\":"+action+",\"x\":"+x+",\"y\":"+y+",\"hp\":"+(byte)read(e,V+0x3a,1)+"}");
    }finally{e.dispose();}
   }
  }
 }
}
