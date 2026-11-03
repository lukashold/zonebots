// Native APC approach/inside/exit counters and group placement. Numeric output
// only; path, palette and final occupancy/object allocation are intercepted.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeApcTransportOracle extends GhidraScript {
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,G=0x60ec00L,W=0x1000000L,O=W+0x1000;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void invoke(EmulatorHelper e,long a)throws Exception{e.writeRegister("ESP",S);put(e,S,4,RET);put(e,S+4,4,U);e.writeRegister("ECX",U);if(!e.run(toAddr(a),null,monitor))throw new Exception(e.getLastError());}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int direction=0;direction<8;direction++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{e.writeMemory(toAddr(U),new byte[0xc0]);put(e,U+0x39,1,direction);put(e,U+0x35,1,61);put(e,U+0x3c,1,2);put(e,U+0x69,1,4);put(e,U+0x2c,2,500);e.setBreakpoint(toAddr(RET));
    for(int tick=1;tick<=24;tick++){invoke(e,0x456c00);int state=(int)read(e,U+0x35,1);out.println("{\"kind\":\"enter\",\"direction\":"+direction+",\"tick\":"+tick+",\"state\":"+state+",\"frame\":"+read(e,U+0x32,2)+",\"timer\":"+(byte)read(e,U+0x3c,1)+",\"visible\":"+((read(e,U+0x69,1)&4)!=0)+",\"x\":"+read(e,U+0x2c,2)+"}");if(state==62)break;}
   }finally{e.dispose();}
  }
  for(int direction=0;direction<8;direction++)for(int rank:new int[]{0,1,5,20})for(int multi=0;multi<2;multi++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{e.writeMemory(toAddr(U),new byte[0xc0]);e.writeMemory(toAddr(G),new byte[0x48]);put(e,0x632194,4,G);put(e,U+0x39,1,direction);put(e,U+0x35,1,64);put(e,U+0x3c,1,5+rank*6);put(e,U+0x8a,1,32);put(e,U+0x2c,2,500);put(e,U+0x2e,2,600);put(e,0x622dc4,1,multi);for(int i=0;i<5;i++)put(e,0x622cdc+i*4,4,i+1);
    for(long a:new long[]{RET,0x452500,0x41b950,0x454070,0x4525a0})e.setBreakpoint(toAddr(a));
    for(int tick=1;tick<=160;tick++){invoke(e,0x456f20);boolean release=false;int x=0,y=0;
     while(e.readRegister("EIP").longValue()!=RET){long a=e.readRegister("EIP").longValue(),s=e.readRegister("ESP").longValue();if(a==0x4525a0){release=true;x=e.readRegister("EDX").intValue()&65535;y=(int)read(e,s+4,4);}else if(a==0x454070)e.writeRegister("EAX",0);else if(a!=0x452500&&a!=0x41b950)throw new Exception("Unexpected exit stop");resume(e);}
     out.println("{\"kind\":\"exit\",\"direction\":"+direction+",\"rank\":"+rank+",\"multi\":"+multi+",\"tick\":"+tick+",\"frame\":"+read(e,U+0x32,2)+",\"timer\":"+(byte)read(e,U+0x3c,1)+",\"visible\":"+((read(e,U+0x69,1)&4)!=0)+",\"release\":"+release+",\"x\":"+x+",\"y\":"+y+"}");if(release)break;
    }
   }finally{e.dispose();}
  }
  for(int direction=0;direction<8;direction++)for(int count:new int[]{1,6,21}){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{e.writeMemory(toAddr(U),new byte[0xc0*32]);e.writeMemory(toAddr(G),new byte[0x48]);e.writeMemory(toAddr(W),new byte[0x2000]);put(e,U+8,4,W);put(e,W+0x27,1,18);put(e,0x4d1918,4,O);put(e,O,2,500);put(e,O+2,2,600);put(e,U+0x39,1,direction);put(e,U+0x8d,1,4);put(e,U+0x8e,1,0);
    for(int i=0;i<21;i++){put(e,G+0x13+i,1,i<count?i:255);put(e,U+i*0xc0+0x8a,1,56);}for(long a:new long[]{RET,0x458af0})e.setBreakpoint(toAddr(a));invoke(e,0x459720);while(e.readRegister("EIP").longValue()!=RET){if(e.readRegister("EIP").longValue()!=0x458af0)throw new Exception("Unexpected group unload stop");resume(e);}
    for(int i=0;i<count;i++){long v=U+i*0xc0;out.println("{\"kind\":\"placement\",\"direction\":"+direction+",\"count\":"+count+",\"rank\":"+i+",\"facing\":"+read(e,v+0x39,1)+",\"state\":"+read(e,v+0x35,1)+",\"frame\":"+read(e,v+0x32,2)+",\"timer\":"+(byte)read(e,v+0x3c,1)+",\"x\":"+read(e,v+0x2c,2)+",\"y\":"+read(e,v+0x2e,2)+"}");}
   }finally{e.dispose();}
  }
  for(int count:new int[]{1,6,21})for(int inside=0;inside<=count;inside++)for(int occupied=0;occupied<2;occupied++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{e.writeMemory(toAddr(U),new byte[0xc0*240]);e.writeMemory(toAddr(G),new byte[0x48]);put(e,0x632194,4,G);for(int i=0;i<21;i++){put(e,G+0x13+i,1,i<count?i:255);if(i<count)put(e,U+i*0xc0+0x8a,1,i<inside?32:0);}put(e,U+0x35,1,occupied!=0?56:62);put(e,U+0x36,2,1);for(int i=0;i<240;i++)put(e,U+i*0xc0+0xb5,1,255);for(long a:new long[]{RET,0x456c90})e.setBreakpoint(toAddr(a));invoke(e,0x456d50);boolean transform=e.readRegister("EIP").longValue()==0x456c90;out.println("{\"kind\":\"completion\",\"count\":"+count+",\"inside\":"+inside+",\"occupied\":"+occupied+",\"transform\":"+transform+"}");
   }finally{e.dispose();}
  }
 }}
}
