// Execute native capture-arrival transitions with movement and pickup intercepted.
// Only generated numerical fixtures/results are written.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeArrivalTaskOracle extends GhidraScript {
 static final long RET=0x12345000L,STACK=0x7fff0000L,SECT=0x5c3f0cL,GROUP=0x5c47b8L,UNIT=0x5e30c8L,ROSTER=0x60ec00L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void invoke(EmulatorHelper e,long a,long g)throws Exception{e.writeRegister("ESP",STACK);put(e,STACK,4,RET);e.writeRegister("ECX",g);if(!e.run(toAddr(a),null,monitor))throw new Exception(e.getLastError());}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int scenario=0;scenario<64;scenario++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{
    e.writeMemory(toAddr(GROUP),new byte[240*24]);e.writeMemory(toAddr(ROSTER),new byte[240*72]);e.writeMemory(toAddr(UNIT),new byte[240*192]);e.writeMemory(toAddr(SECT),new byte[4*30]);
    for(int s=0;s<4;s++)put(e,SECT+s*30+8,1,(scenario&1)!=0&&s%2==0?1:0);
    int[]tasks={1,0,2,3,6,4,5,-1};
    for(int i=0;i<8;i++){
     long g=GROUP+i*24,u=UNIT+i*192,r=ROSTER+i*72;int target=i%2;
     put(e,g,4,SECT+((scenario&2)!=0&&i==2?3:target)*30);put(e,g+4,4,SECT+target*30);put(e,g+12,4,(scenario&4)!=0&&i==4?0xffffffffL:SECT+target*30);put(e,g+16,1,24);put(e,g+17,1,tasks[(i+scenario/8)%8]);put(e,g+21,1,(scenario&8)==0||i%2==0?1:0);
     for(int m=0;m<21;m++)put(e,r+19+m,1,m==0?i:255);put(e,r+16,1,(scenario&16)!=0&&i%3==0?1:0);put(e,u+0x31,1,(scenario&32)!=0&&i==2?0:1);
    }
    for(long a:new long[]{RET,0x414830,0x414720,0x414a10})e.setBreakpoint(toAddr(a));
    for(int n=0;n<8;n++){
     // This is the15000 outer active/mobile/budget gate.
     long g=GROUP+n*24;if(read(e,g+21,1)==0)continue;
     invoke(e,0x414a80,g);
     while(e.readRegister("EIP").longValue()!=RET){long a=e.readRegister("EIP").longValue();
      if(a==0x414a10)out.println("{\"kind\":\"capture\",\"scenario\":"+scenario+",\"group\":"+n+",\"member\":"+((e.readRegister("EDX").longValue()-UNIT)/192)+",\"sector\":"+((e.readRegister("ECX").longValue()-SECT)/30)+"}");
      else if(a==0x414830)out.println("{\"kind\":\"captured\",\"scenario\":"+scenario+",\"group\":"+n+",\"sector\":"+((read(e,g+12,4)-SECT)/30)+"}");
      else if(a!=0x414720)throw new Exception("Unexpected arrival stop");resume(e);
     }
    }
    for(int i=0;i<8;i++){long g=GROUP+i*24,via=read(e,g+4,4),assigned=read(e,g+12,4);out.println("{\"kind\":\"group\",\"scenario\":"+scenario+",\"group\":"+i+",\"task\":"+(byte)read(e,g+17,1)+",\"via\":"+(via==0xffffffffL?-1:(via-SECT)/30)+",\"assigned\":"+(assigned==0xffffffffL?-1:(assigned-SECT)/30)+"}");}
   }finally{e.dispose();}
  }
 }}
}
