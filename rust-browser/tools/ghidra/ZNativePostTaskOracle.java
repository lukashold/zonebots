// Native post15000 phases with bounded generated worlds and intercepted commands.
// No original resource data or instruction bytes are written.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativePostTaskOracle extends GhidraScript {
 static final long RET=0x12345000L,STACK=0x7fff0000L,SECT=0x5c3f0cL,GROUP=0x5c47b8L,UNIT=0x5e30c8L,ROSTER=0x60ec00L,BUILD=0x5c5e38L,OBJECT=0x612f80L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void invoke(EmulatorHelper e,long a)throws Exception{e.writeRegister("ESP",STACK);put(e,STACK,4,RET);if(!e.run(toAddr(a),null,monitor))throw new Exception(e.getLastError());}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 void init(EmulatorHelper e)throws Exception{
  e.writeMemory(toAddr(GROUP),new byte[240*24]);e.writeMemory(toAddr(ROSTER),new byte[240*72]);e.writeMemory(toAddr(UNIT),new byte[240*192]);e.writeMemory(toAddr(BUILD),new byte[35*88]);e.writeMemory(toAddr(OBJECT),new byte[200*44]);e.writeMemory(toAddr(0x5c6a44),new byte[240*20]);
  for(int i=0;i<64;i++){e.writeMemory(toAddr(SECT+i*30),new byte[30]);put(e,SECT+i*30,2,65535);}
  for(int i=0;i<35;i++)put(e,BUILD+i*88+4,1,255);
  for(int i=0;i<240;i++){put(e,0x5c6a44+i*20+10,1,255);for(int m=0;m<21;m++)put(e,ROSTER+i*72+19+m,1,255);}
  for(int i=0;i<8;i++){long g=GROUP+i*24,u=UNIT+i*192,r=ROSTER+i*72;put(e,g,4,SECT);put(e,g+4,4,SECT);put(e,g+12,4,SECT);put(e,g+16,1,24);put(e,g+17,1,255);put(e,g+21,1,1);put(e,r+19,1,i);put(e,r+18,1,1);put(e,u+0x31,1,5);put(e,u+0x30,1,4);put(e,u+0x5e,1,255);put(e,u+0x5f,1,i);put(e,u+0x2c,2,32+i*16);put(e,u+0x2e,2,64);}
  put(e,0x622dc4,1,0);e.setBreakpoint(toAddr(RET));
 }
 void sectors(EmulatorHelper e,int n)throws Exception{for(int i=0;i<n;i++){long s=SECT+i*30;put(e,s,2,i*8);put(e,s+2,2,0);put(e,s+4,2,i*8+8);put(e,s+6,2,8);put(e,s+18,1,i*8+4);put(e,s+19,1,4);}}
 void states(PrintWriter out,EmulatorHelper e,int phase,int scenario)throws Exception{for(int i=0;i<8;i++){long g=GROUP+i*24,via=read(e,g+4,4),assigned=read(e,g+12,4);out.println("{\"kind\":\"group\",\"phase\":"+phase+",\"scenario\":"+scenario+",\"group\":"+i+",\"task\":"+(byte)read(e,g+17,1)+",\"via\":"+(via==0xffffffffL?-1:(via-SECT)/30)+",\"assigned\":"+(assigned==0xffffffffL?-1:(assigned-SECT)/30)+",\"active\":"+read(e,g+21,1)+",\"counter\":"+(byte)read(e,g+18,1)+"}");}}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int phase=0;phase<5;phase++)for(int scenario=0;scenario<64;scenario++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);int rng=123+scenario,calls=0,order=0;
   try{
    init(e);sectors(e,phase==4?7:4);
    if(phase==0){
     int[]types={5,8,25,26,37,10,21,35};
     for(int i=0;i<8;i++){long g=GROUP+i*24,o=OBJECT+i*44;put(e,g+4,4,SECT+(i%4)*30);put(e,g+12,4,SECT+90);put(e,g+17,1,(scenario&16)!=0&&i%2==1?4:5);put(e,g+21,1,(scenario&32)==0||i%3!=0?1:0);int s=i%4;put(e,o,2,scenario%4==0&&i%3==0?0:s*128+32);put(e,o+2,2,64);put(e,o+4,4,(scenario&8)!=0&&i%2==0?0:0x498f38+types[(scenario+i)%8]*26);}
     invoke(e,0x4152c0);
    }else if(phase==1){
     for(int i=0;i<4;i++)put(e,SECT+i*30+8,1,(scenario+i)%3==0?1:0);
     for(int i=0;i<8;i++){long u=UNIT+i*192;put(e,u+0x2c,2,(i%4)*128+32);put(e,u+0x2e,2,64);put(e,u+0x31,1,(scenario+i)%4);if((scenario+i)%3!=0)put(e,u+0x31,1,read(e,u+0x31,1)|4);put(e,u+0x8a,1,(scenario*7+i*3)&31);}
     e.setBreakpoint(toAddr(0x44e910));invoke(e,0x4158f0);
     while(e.readRegister("EIP").longValue()!=RET){if(e.readRegister("EIP").longValue()!=0x44e910)throw new Exception("Unexpected rejoin stop");out.println("{\"kind\":\"rejoin\",\"phase\":"+phase+",\"scenario\":"+scenario+",\"unit\":"+((e.readRegister("ECX").longValue()-UNIT)/192)+"}");resume(e);}
     for(int i=0;i<8;i++)out.println("{\"kind\":\"unit\",\"phase\":"+phase+",\"scenario\":"+scenario+",\"unit\":"+i+",\"flags\":"+read(e,UNIT+i*192+0x8a,1)+"}");
    }else if(phase==2){
     int[]classes={0,16,15,3,1,1,0,1},families={4,4,4,4,6,34,4,24};
     for(int i=0;i<4;i++){put(e,SECT+i*30+8,1,(scenario&2)!=0&&i==2?1:0);put(e,SECT+i*30+24,1,(scenario&4)!=0&&i==3?2:0);long b=BUILD+i*88;put(e,b,2,i*128+16);put(e,b+2,2,32);put(e,b+4,1,(scenario&8)!=0&&i==1?255:i);put(e,b+32,4,SECT+i*30);put(e,0x498c78+i*76+50,1,-7-i);put(e,0x498c78+i*76+51,1,23+i);}
     for(int i=0;i<8;i++){long g=GROUP+i*24,u=UNIT+i*192;put(e,g,4,SECT+(i%4)*30);put(e,g+12,4,SECT+(i%4)*30);put(e,g+17,1,i%2==0?1:3);put(e,u+0x7c,1,classes[i]);put(e,u+0x30,1,families[i]);}
     for(long a:new long[]{0x411b90,0x456640,0x45ef00})e.setBreakpoint(toAddr(a));invoke(e,0x4150f0);
     while(e.readRegister("EIP").longValue()!=RET){long a=e.readRegister("EIP").longValue();
      if(a==0x411b90)e.writeRegister("EAX",(scenario&1)!=0?1:0);
      else if(a==0x456640){int x=(short)e.readRegister("ECX").intValue(),y=(short)e.readRegister("EDX").intValue();put(e,0x62d3a8,1,16);put(e,0x62d3a9,1,255);put(e,UNIT+16*192+0x31,1,(scenario&32)!=0?1:0);put(e,UNIT+16*192+0x2c,2,x+((scenario&16)!=0?129:64));put(e,UNIT+16*192+0x2e,2,y+64);}
      else if(a==0x45ef00)out.println("{\"kind\":\"attack\",\"phase\":"+phase+",\"scenario\":"+scenario+",\"order\":"+order+++",\"member\":"+((read(e,0x62dc04,4)-UNIT)/192)+",\"x\":"+(short)read(e,0x62dc3c,2)+",\"y\":"+(short)read(e,0x62dc48,2)+"}");
      else throw new Exception("Unexpected building stop");resume(e);
     }
    }else if(phase==3){
     int[]types={5,8,25,26,37,10,21,35};
     for(int i=0;i<4;i++)put(e,SECT+i*30+29,1,(scenario&1)!=0&&i%2!=0?1:0);
     for(int i=0;i<8;i++){long g=GROUP+i*24,u=UNIT+i*192;put(e,g+4,4,SECT+(i%4)*30);put(e,g+12,4,SECT+90);put(e,g+17,1,i>=5?5:255);put(e,u+0x7c,1,i==4?1:i%4==0?0:i%4==1?16:i%4==2?3:15);put(e,u+0x9c,1,(scenario&2)!=0&&i==2?4:0);put(e,u+0x5e,1,(scenario&4)!=0&&i==1?0:255);long o=OBJECT+i*44;put(e,o,2,(i%4)*128+32);put(e,o+2,2,64);put(e,o+4,4,0x498f38+types[(scenario+i)%8]*26);put(e,o+13,1,(scenario&8)!=0&&i%3==0?1:0);put(e,o+26,1,(scenario&16)!=0&&i%3==1?1:0);}
     e.setBreakpoint(toAddr(0x460910));e.setBreakpoint(toAddr(0x45ef00));invoke(e,0x415c00);
     while(e.readRegister("EIP").longValue()!=RET){long a=e.readRegister("EIP").longValue();if(a==0x460910){rng=rng*1664525+1013904223;calls++;e.writeRegister("EAX",rng);}else if(a==0x45ef00)out.println("{\"kind\":\"support\",\"phase\":"+phase+",\"scenario\":"+scenario+",\"order\":"+order+++",\"member\":"+((read(e,0x62dc04,4)-UNIT)/192)+",\"x\":"+(short)read(e,0x62dc00,2)+",\"y\":"+(short)read(e,0x62dc3c,2)+"}");else throw new Exception("Unexpected support stop");resume(e);}
    }else{
     int[]classes={0,1,16,15,3,0,1,2};
     for(int i=0;i<7;i++){put(e,SECT+i*30+8,1,i==4||i==6?0:1);put(e,SECT+i*30+24,1,i==4&&(scenario&8)!=0?32:i==6&&(scenario&16)!=0?64:0);}
     for(int i=0;i<8;i++){long g=GROUP+i*24,u=UNIT+i*192,r=ROSTER+i*72;put(e,g,4,SECT+(i%2)*30);put(e,g+4,4,SECT+90);put(e,g+12,4,SECT+60);put(e,g+18,1,i%3);put(e,u+0x7c,1,classes[i]);put(e,r+58,1,(scenario&1)!=0&&i==3?1:0);if((scenario&32)!=0&&i==1)put(e,r+19,1,255);}
     if((scenario&2)!=0){put(e,0x5c6a44,4,ROSTER+2*72);put(e,0x5c6a44+10,1,0);}
     for(long a:new long[]{0x460910,0x45f070,0x45ebd0,0x411b90})e.setBreakpoint(toAddr(a));invoke(e,0x415790);
     while(e.readRegister("EIP").longValue()!=RET){long a=e.readRegister("EIP").longValue();if(a==0x460910){rng=rng*1664525+1013904223;calls++;e.writeRegister("EAX",rng);}else if(a==0x411b90)e.writeRegister("EAX",(scenario&4)!=0?1:0);else if(a==0x45f070||a==0x45ebd0)out.println("{\"kind\":\"frontier\",\"phase\":"+phase+",\"scenario\":"+scenario+",\"order\":"+order+++",\"member\":"+((read(e,0x62dc04,4)-UNIT)/192)+",\"x\":"+(short)read(e,0x62dc00,2)+",\"y\":"+(short)read(e,0x62dc3c,2)+"}");else throw new Exception("Unexpected frontier stop");resume(e);}
    }
    states(out,e,phase,scenario);out.println("{\"kind\":\"rng\",\"phase\":"+phase+",\"scenario\":"+scenario+",\"calls\":"+calls+"}");
   }finally{e.dispose();}
  }
 }}
}
