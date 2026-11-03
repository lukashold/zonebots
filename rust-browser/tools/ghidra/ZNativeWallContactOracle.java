// Executes native signed-byte wall HP and tall-column normalization.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
public class ZNativeWallContactOracle extends GhidraScript {
 static final long RET=0x12345000L,S=0x7fff0000L,T=0x58aaaaL,W=0x1100000L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void run(EmulatorHelper e,int strength,int cell)throws Exception{
  e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",strength);e.writeRegister("EDX",cell);e.setBreakpoint(toAddr(RET));
  if(!e.run(toAddr(0x42fdb0),null,monitor))throw new Exception(e.getLastError());
  while(e.readRegister("EIP").longValue()!=RET){long pc=e.readRegister("EIP").longValue(),stack=e.readRegister("ESP").longValue();if(pc==0x42f660)e.writeRegister("EAX",1);else if(pc!=0x42f580&&pc!=0x42f1f0&&pc!=0x42f720)throw new Exception("Unexpected "+Long.toHexString(pc));long ret=read(e,stack,4);e.writeRegister("ESP",stack+4);if(!e.run(toAddr(ret),null,monitor))throw new Exception(e.getLastError());}
 }
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{
  for(long a:new long[]{0x42f580,0x42f660,0x42f1f0,0x42f720})e.setBreakpoint(toAddr(a));put(e,0x63249c,4,W);e.writeMemory(toAddr(T),new byte[16384]);e.writeMemory(toAddr(0x5c8fc0),new byte[138]);
  byte[]pool=new byte[5000*6];for(int i=0;i<5000;i++){pool[i*6]=(byte)255;pool[i*6+1]=(byte)255;}
  int s=0;for(int hp=-128;hp<128;hp++)for(int strength:new int[]{0,1,50,127,128,255}){
   e.writeMemory(toAddr(W),pool);put(e,W,2,500);put(e,W+2,1,0);put(e,W+3,1,hp);run(e,strength,500);out.println("{\"kind\":\"wallDamage\",\"scenario\":"+s+++",\"hp\":"+read(e,W+3,1)+",\"dead\":"+(read(e,W,2)==65535?1:0)+"}");
  }
  for(s=0;s<64;s++){int head=20*128+20,height=s/8%5,cell=head+((s&7)-1)*128;e.writeMemory(toAddr(T),new byte[16384]);put(e,T+head,1,3);for(int j=1;j<=height;j++)put(e,T+head+j*128,1,18);e.writeMemory(toAddr(W),pool);put(e,W,2,head);put(e,W+2,1,0);put(e,W+3,1,10);run(e,1,cell);out.println("{\"kind\":\"wallAnchor\",\"scenario\":"+s+",\"hp\":"+read(e,W+3,1)+",\"height\":"+read(e,0x4a897b,1)+"}");}
 }finally{e.dispose();}}}
}
