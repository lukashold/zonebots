// Generated numeric lifecycle fixtures:17a50 initial sector bits,161b0 pickup
// restriction release,15d20 totals,15450 stalled-task signed counters.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeTaskLifecycleOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,G=0x5c47b8L,B=0x5c3f0cL,U=0x5e30c8L,R=0x60ec00L,O=0x612f80L,D=0x5c5e38L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void invoke(EmulatorHelper e,long a,long ecx)throws Exception{e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",ecx);if(!e.run(toAddr(a),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int scenario=0;scenario<64;scenario++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{
    e.writeMemory(toAddr(D),new byte[35*88]);e.writeMemory(toAddr(O),new byte[200*44]);
    for(int i=0;i<64;i++){e.writeMemory(toAddr(B+i*30),new byte[30]);put(e,B+i*30,2,65535);}
    for(int i=0;i<4;i++){long b=B+i*30;put(e,b,2,i*8);put(e,b+4,2,i*8+8);put(e,b+6,2,8);put(e,b+24,1,(scenario*13+i*37)&255);}
    for(int i=0;i<35;i++)put(e,D+i*88+4,1,255);
    put(e,D,2,160);put(e,D+2,2,64);put(e,D+4,1,0);put(e,D+5,1,0);put(e,D+30,2,(scenario&1)!=0?0:100);
    int[]defs={5,26,25,37};
    for(int i=0;i<4;i++){long o=O+i*44;put(e,o,2,(scenario&2)!=0&&i==0?0:i*128+32);put(e,o+2,2,64);put(e,o+4,4,(scenario&8)!=0&&i==2?0:0x498f38+defs[i]*26);put(e,o+26,1,7+i);}
    for(int i=0;i<4;i++){put(e,0x622d54+i*4,2,i==2&&(scenario&16)==0||i==3&&(scenario&32)==0?-512:i*128+64);put(e,0x622d56+i*4,2,64);}
    e.setBreakpoint(toAddr(RET));invoke(e,0x417a50,0);
    for(int i=0;i<4;i++)out.println("{\"kind\":\"initial\",\"scenario\":"+scenario+",\"sector\":"+i+",\"flags\":"+read(e,B+i*30+24,1)+",\"byte21\":"+read(e,B+i*30+21,1)+"}");
    put(e,0x622dc4,1,0);put(e,0x622c7a,2,(scenario&4)!=0?18:17);invoke(e,0x4161b0,B+(scenario%4)*30);
    for(int i=0;i<4;i++)out.println("{\"kind\":\"object\",\"scenario\":"+scenario+",\"object\":"+i+",\"flags\":"+read(e,O+i*44+26,1)+"}");
   }finally{e.dispose();}
  }
  for(int scenario=0;scenario<64;scenario++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{
    e.writeMemory(toAddr(G),new byte[240*24]);e.writeMemory(toAddr(U),new byte[240*192]);e.writeMemory(toAddr(R),new byte[240*72]);e.writeMemory(toAddr(0x5c6a44),new byte[240*20]);
    for(int i=0;i<240;i++){put(e,0x5c6a44+i*20+10,1,255);for(int m=0;m<21;m++)put(e,R+i*72+19+m,1,255);}
    int[]tasks={-1,0,1,2,3,4,5,6},counts={0,1,2,3,127,-1,-128,0};
    for(int i=0;i<8;i++){long g=G+i*24,u=U+i*192,r=R+i*72;put(e,g+4,4,B+30);put(e,g+12,4,B+60);put(e,g+16,1,(scenario&2)!=0&&i==1?8:(scenario&4)!=0&&i==2?16:24);put(e,g+17,1,tasks[(scenario+i)%8]);put(e,g+20,1,counts[i]);put(e,g+21,1,(scenario&1)==0||i!=6?1:0);put(e,r+19,1,i);put(e,r+58,1,i==5?1:0);put(e,u+0x31,1,(scenario&8)!=0&&i==3?0:5|((scenario&16)!=0&&i==4?32:0));put(e,u+0x14,4,i==4?0x70000000:0);put(e,u+12,4,0);}
    if((scenario&32)!=0){put(e,0x5c6a44,4,R+6*72);put(e,0x5c6a44+10,1,0);}
    e.setBreakpoint(toAddr(RET));
    for(int tick=1;tick<=8;tick++){
     invoke(e,0x415450,0);
     for(int i=0;i<8;i++){long g=G+i*24,via=read(e,g+4,4),assigned=read(e,g+12,4);out.println("{\"kind\":\"retry\",\"scenario\":"+scenario+",\"tick\":"+tick+",\"group\":"+i+",\"task\":"+(byte)read(e,g+17,1)+",\"via\":"+(via==0xffffffffL?-1:(via-B)/30)+",\"assigned\":"+(assigned==0xffffffffL?-1:(assigned-B)/30)+",\"counter\":"+(byte)read(e,g+20,1)+"}");}
    }
   }finally{e.dispose();}
  }
 }}
}
