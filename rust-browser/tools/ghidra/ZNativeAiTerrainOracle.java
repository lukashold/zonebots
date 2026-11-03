// Numeric native AI grenade capability, reserve release and terrain flood.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeAiTerrainOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,G=0x5c47b8L,B=0x5c3f0cL,MALLOC=RET+0x100,FREE=RET+0x200;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void invoke(EmulatorHelper e,long a,long ecx)throws Exception{e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",ecx);e.setBreakpoint(toAddr(RET));if(!e.run(toAddr(a),null,monitor))throw new Exception(e.getLastError());}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 byte[] terrain(int scenario){byte[]a=new byte[16384];java.util.Arrays.fill(a,(byte)1);int mode=scenario%8,radius=1+scenario/8*3;
  if(mode==0)return a;
  if(mode==3){java.util.Arrays.fill(a,(byte)0);return a;}
  int r=mode==4?35:radius;for(int y=64-r;y<=64+r;y++)for(int x=64-r;x<=64+r;x++)a[y*128+x]=0;
  if(mode==2)for(int y=0;y<=64;y++)a[y*128+64]=0;
  if(mode==5||mode==6)a[64*128+65]=35;return a;
 }
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int scenario=0;scenario<256;scenario++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{e.writeMemory(toAddr(U),new byte[0xc0*21]);e.writeMemory(toAddr(0x60ec00),new byte[0x48]);
    for(int i=0;i<21;i++){put(e,0x60ec13+i,1,((scenario>>(i%8))&1)!=0?i:255);put(e,U+i*0xc0+0x99,1,scenario*37+i*17);}
    invoke(e,0x411b90,U);out.println("{\"kind\":\"grenades\",\"scenario\":"+scenario+",\"sum\":"+(e.readRegister("EAX").longValue()&65535)+"}");
   }finally{e.dispose();}
  }
  for(int scenario=0;scenario<32;scenario++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{e.writeMemory(toAddr(G),new byte[24*240]);e.writeMemory(toAddr(0x60ec00),new byte[0x48*240]);e.writeMemory(toAddr(U),new byte[0xc0*8]);e.writeMemory(toAddr(B),new byte[30*4]);
    put(e,B+24,1,1);put(e,B+30+24,1,2);put(e,0x4a84c4,4,B);
    int[]tasks={-1,0,1,2,3,4,5,6};
    for(int i=0;i<8;i++){long g=G+i*24;
     put(e,g+21,1,(scenario&16)==0||i%2==0?1:0);put(e,g+17,1,tasks[(i+scenario)&7]);put(e,g+16,1,8*(1+i%3));
     put(e,g,4,B+((scenario&1)!=0&&i%2==0?30:60));put(e,g+12,4,(scenario&4)!=0&&i%3==0?0:B+(((scenario+i)&2)!=0?60:90));
     put(e,0x60ec13+i*0x48,1,(scenario&8)!=0&&i%3==0?255:i);put(e,U+i*0xc0+0x31,1,(scenario&2)!=0&&i%3==0?4:5);
    }
    e.setBreakpoint(toAddr(0x419430));
    for(int released=0;released<8;released++){
     invoke(e,0x416d70,G+released*24);
     while(e.readRegister("EIP").longValue()!=RET){long a=e.readRegister("EIP").longValue(),s=e.readRegister("ESP").longValue();if(a!=0x419430)throw new Exception("Unexpected release stop");
      out.println("{\"kind\":\"fallback\",\"scenario\":"+scenario+",\"released\":"+released+",\"leader\":"+((e.readRegister("EDX").longValue()-U)/0xc0)+",\"metadata\":"+e.readRegister("ECX").intValue()+",\"sector\":"+(read(e,s+4,4)==B?0:-1)+"}");resume(e);
     }
     for(int i=0;i<8;i++)out.println("{\"kind\":\"releaseGroup\",\"scenario\":"+scenario+",\"released\":"+released+",\"group\":"+i+",\"flags\":"+read(e,G+i*24+16,1)+",\"task\":"+(byte)read(e,G+i*24+17,1)+"}");
    }
   }finally{e.dispose();}
  }
  for(int scenario=0;scenario<32;scenario++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{e.writeMemory(toAddr(B),new byte[30*64]);for(int i=0;i<64;i++)put(e,B+i*30,2,65535);put(e,B,2,1);put(e,B+18,1,64);put(e,B+19,1,64);put(e,B+24,1,scenario%8==7?66:64);e.writeMemory(toAddr(0x58aaaa),terrain(scenario));
    put(e,0x474078,4,MALLOC);put(e,0x474058,4,FREE);for(long a:new long[]{MALLOC,FREE,0x417400})e.setBreakpoint(toAddr(a));long heap=0x1100000;
    invoke(e,0x4170a0,U);
    while(e.readRegister("EIP").longValue()!=RET){long a=e.readRegister("EIP").longValue(),s=e.readRegister("ESP").longValue();
     if(a==MALLOC){int size=(int)read(e,s+4,4);e.writeMemory(toAddr(heap),new byte[size]);e.writeRegister("EAX",heap);heap+=size+0x100;}
     else if(a==FREE)e.writeRegister("EAX",0);
     else if(a==0x417400){int x=(short)e.readRegister("ECX").intValue();e.writeRegister("EAX",scenario%8==5&&x>1024?B+30:B);}
     else throw new Exception("Unexpected flood stop "+Long.toHexString(a));resume(e);
    }
    out.println("{\"kind\":\"flood\",\"scenario\":"+scenario+",\"flags\":"+read(e,B+24,1)+"}");
   }finally{e.dispose();}
  }
 }}
}
