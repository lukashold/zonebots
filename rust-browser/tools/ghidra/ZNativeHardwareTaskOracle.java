// Execute14830/17900/16070; the native follower allocator is intercepted and
// returns null so the original's explicit leader fallback is exercised.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeHardwareTaskOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,G=0x5c47b8L,B=0x5c3f0cL,U=0x5e30c8L,R=0x60ec00L,O=0x612f80L,D=0x5c5e38L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int scenario=0;scenario<64;scenario++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);int order=0,split=0;
   try{
    e.writeMemory(toAddr(G),new byte[240*24]);e.writeMemory(toAddr(U),new byte[240*192]);e.writeMemory(toAddr(R),new byte[240*72]);e.writeMemory(toAddr(D),new byte[35*88]);e.writeMemory(toAddr(O),new byte[200*44]);
    for(int i=0;i<64;i++){e.writeMemory(toAddr(B+i*30),new byte[30]);put(e,B+i*30,2,65535);}
    for(int i=0;i<4;i++){put(e,B+i*30,2,i*8);put(e,B+i*30+2,2,0);put(e,B+i*30+4,2,i*8+8);put(e,B+i*30+6,2,8);put(e,B+i*30+8,1,1);put(e,B+i*30+24,1,(scenario&1)==0?16:0);}
    for(int i=0;i<35;i++)put(e,D+i*88+4,1,255);
    put(e,D,2,16);put(e,D+2,2,32);put(e,D+4,1,0);put(e,D+5,1,0);put(e,D+30,2,(scenario&2)==0?100:0);
    int[]defs={5,26,8};for(int i=0;i<3;i++){put(e,O+i*44,2,(scenario&8)!=0&&i==0?0:32+i*16);put(e,O+i*44+2,2,64);put(e,O+i*44+4,4,0x498f38+defs[i]*26);put(e,O+i*44+26,1,i==0&&(scenario&4)!=0?1:0);}
    for(int i=0;i<8;i++){long g=G+i*24,u=U+i*192,r=R+i*72;put(e,g,4,B);put(e,g+4,4,B);put(e,g+12,4,B+90);put(e,g+16,1,i==6?28:24);put(e,g+17,1,i==1||i==2?5:i==3?4:i==4||i==5?3:255);put(e,g+21,1,i==3&&(scenario%5)==0?0:1);put(e,r+18,1,i==4?2:1);
     for(int m=0;m<21;m++)put(e,r+19+m,1,m==0?i:m==1&&i==4?8:255);put(e,u+0x31,1,5);put(e,u+0x30,1,4);put(e,u+0x7c,1,i==0&&(scenario&16)!=0?1:i==1?((scenario&32)!=0?1:0):i==2||i==7?2:i==3?3:0);put(e,u+0x5e,1,255);put(e,u+0x5f,1,i);put(e,u+0x2c,2,16+i*8);put(e,u+0x2e,2,64);
    }
    put(e,0x622dc4,1,0);for(long a:new long[]{RET,0x411b90,0x44c0b0,0x45ef00})e.setBreakpoint(toAddr(a));
    for(int n=0;n<8;n++){
     long g=G+n*24;if(read(e,g+21,1)==0||(read(e,g+16,1)&24)!=24)continue;
     e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",U+n*192);e.writeRegister("EDX",g);if(!e.run(toAddr(0x414830),null,monitor))throw new Exception(e.getLastError());
     while(e.readRegister("EIP").longValue()!=RET){long a=e.readRegister("EIP").longValue();if(a==0x411b90)e.writeRegister("EAX",scenario%3!=0?1:0);
      else if(a==0x44c0b0){split++;e.writeRegister("EAX",0);}
      else if(a==0x45ef00)out.println("{\"kind\":\"order\",\"scenario\":"+scenario+",\"order\":"+order+++",\"member\":"+((read(e,0x62dc04,4)-U)/192)+",\"x\":"+(short)read(e,0x62dc00,2)+",\"y\":"+(short)read(e,0x62dc3c,2)+"}");
      else throw new Exception("Unexpected hardware stop");resume(e);
     }
    }
    out.println("{\"kind\":\"split\",\"scenario\":"+scenario+",\"calls\":"+split+"}");
    for(int i=0;i<8;i++){long g=G+i*24,via=read(e,g+4,4),assigned=read(e,g+12,4);out.println("{\"kind\":\"group\",\"scenario\":"+scenario+",\"group\":"+i+",\"task\":"+(byte)read(e,g+17,1)+",\"via\":"+(via==0xffffffffL?-1:(via-B)/30)+",\"assigned\":"+(assigned==0xffffffffL?-1:(assigned-B)/30)+",\"flags\":"+read(e,g+16,1)+",\"active\":"+read(e,g+21,1)+"}");}
   }finally{e.dispose();}
  }
 }}
}
