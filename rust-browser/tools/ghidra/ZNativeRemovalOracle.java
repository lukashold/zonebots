// Execute58660 allocation and563c0's independent15-record sprite lifecycle.
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
public class ZNativeRemovalOracle extends ZNativeReactionsOracle{
 static final long R=0x615420L;
 int seed,calls;
 long hash(byte[] bytes){long h=0x811c9dc5L;for(byte b:bytes)h=((h^(b&255))*16777619)&0xffffffffL;return h;}
 void invoke(EmulatorHelper e,long entry,long ecx)throws Exception{calls=0;e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",ecx);e.setBreakpoint(toAddr(RET));if(!e.run(toAddr(entry),null,monitor))throw new Exception(e.getLastError());while(e.readRegister("EIP").longValue()!=RET){long pc=e.readRegister("EIP").longValue();if(pc!=0x460910)throw new Exception("Unexpected "+Long.toHexString(pc));calls++;e.writeRegister("EAX",seed&0xffffffffL);long stack=e.readRegister("ESP").longValue(),ret=read(e,stack,4);e.writeRegister("ESP",stack+4);if(!e.run(toAddr(ret),null,monitor))throw new Exception(e.getLastError());}}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{
  e.setBreakpoint(toAddr(0x460910));for(int s=0;s<256;s++){e.writeMemory(toAddr(U),new byte[192]);for(int slot=0;slot<15;slot++){put(e,R+slot*8,2,slot<(s>>3)%16?1000+slot:-1);put(e,R+slot*8+2,2,slot*319);put(e,R+slot*8+4,2,500+slot);put(e,R+slot*8+6,2,600-slot);}put(e,U+8,4,-1);put(e,U+0x31,1,(s&1)!=0?4:0);put(e,U+0x64,1,s*13);put(e,U+0x2c,2,500);put(e,U+0x2e,2,600);put(e,0x5849e7,1,(s>>1)&1);put(e,0x4d029f,1,(s>>2)&1);seed=s*2027^0x7731a5c9;invoke(e,0x458660,U);out.println("{\"kind\":\"allocate\",\"scenario\":"+s+",\"calls\":"+calls+",\"active\":"+read(e,U+0x31,1)+",\"hash\":"+hash(e.readMemory(toAddr(R),120))+"}");}
  int[][]points={{160,100},{-16,0},{-15,0},{319,199},{320,200},{32767,-32768},{-32768,32767},{500,500}},cameras={{0,0},{16,16},{-16,-16},{32760,-32760}};
  for(int s=0;s<32;s++){e.writeMemory(toAddr(R),new byte[120]);for(int slot=0;slot<15;slot++)put(e,R+slot*8,2,-1);int[]p=points[s%8],camera=cameras[s/8];int row=s%17;put(e,R,2,read(e,0x4971e8+row*4,2));put(e,R+2,2,read(e,0x4971ea+row*4,2)|(s&3));put(e,R+4,2,p[0]);put(e,R+6,2,p[1]);put(e,0x622cb8,2,camera[0]);put(e,0x622cba,2,camera[1]);put(e,0x498c24,2,320);put(e,0x498c20,2,200);
   for(int tick=1;tick<=40;tick++){put(e,0x622dc0,1,tick&1);put(e,0x62b3a4,4,P);e.writeMemory(toAddr(P),new byte[120]);invoke(e,0x4563c0,0);int count=(int)(read(e,0x62b3a4,4)-P)/8;out.println("{\"kind\":\"advance\",\"scenario\":"+s+",\"tick\":"+tick+",\"hash\":"+hash(e.readMemory(toAddr(R),120))+",\"draws\":"+count+",\"drawHash\":"+(count==0?0x811c9dc5L:hash(e.readMemory(toAddr(P),count*8)))+"}");}
  }
 }finally{e.dispose();}}}
}
