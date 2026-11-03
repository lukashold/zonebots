// Native every-even-callback local hardware goal and signed timer004197d0.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeVehicleTaskOracle extends GhidraScript{
 static final long RET=0x12345000L,STACK=0x7fff0000L,SECT=0x5c3f0cL,GROUP=0x5c47b8L,UNIT=0x5e30c8L,ROSTER=0x60ec00L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int scenario=0;scenario<8;scenario++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{
    e.writeMemory(toAddr(GROUP),new byte[240*24]);e.writeMemory(toAddr(ROSTER),new byte[240*72]);e.writeMemory(toAddr(UNIT),new byte[240*192]);e.writeMemory(toAddr(SECT),new byte[30]);
    put(e,SECT+8,1,(scenario&1)!=0?1:0);put(e,SECT+18,1,4);put(e,SECT+19,1,4);put(e,SECT+24,1,(scenario&2)!=0?2:0);
    int[]families={6,4,5,32,34,36,25,6},classes={1,0,2,1,1,1,1,1},xs={64,64,64,192,191,-64,64,64};
    for(int i=0;i<8;i++){
     long g=GROUP+i*24,u=UNIT+i*192;put(e,g,4,SECT);put(e,g+19,1,i);put(e,g+21,1,i==7&&(scenario&4)!=0?0:1);put(e,ROSTER+i*72+19,1,i);put(e,u+0x31,1,i==6?4:5);put(e,u+0x30,1,families[i]);put(e,u+0x7c,1,classes[i]);put(e,u+0x2c,2,xs[i]);put(e,u+0x2e,2,64);
    }
    for(long a:new long[]{RET,0x4525a0})e.setBreakpoint(toAddr(a));
    for(int tick=1;tick<=110;tick++){
     e.writeRegister("ESP",STACK);put(e,STACK,4,RET);if(!e.run(toAddr(0x4197d0),null,monitor))throw new Exception(e.getLastError());
     while(e.readRegister("EIP").longValue()!=RET){long a=e.readRegister("EIP").longValue(),sp=e.readRegister("ESP").longValue();
      if(a==0x4525a0){int group=(int)((e.readRegister("ECX").longValue()-UNIT)/192),x=e.readRegister("EDX").intValue()&65535,y=(int)read(e,sp+4,4)&65535;out.println("{\"kind\":\"goal\",\"scenario\":"+scenario+",\"tick\":"+tick+",\"group\":"+group+",\"x\":"+x+",\"y\":"+y+"}");e.writeRegister("EAX",0);}
      else throw new Exception("Unexpected hardware task stop");resume(e);
     }
     for(int i=0;i<8;i++)out.println("{\"kind\":\"counter\",\"scenario\":"+scenario+",\"tick\":"+tick+",\"group\":"+i+",\"counter\":"+(byte)read(e,GROUP+i*24+19,1)+"}");
    }
   }finally{e.dispose();}
  }
 }}
}
