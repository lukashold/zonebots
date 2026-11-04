//41d110→41cfa0 full flag/banner queue producer, including offscreen byte5
//progression and cutscene gates. It makes no substituted graphics calls.
import ghidra.app.script.GhidraScript;import ghidra.app.emulator.EmulatorHelper;import java.io.PrintWriter;
public class ZNativeFlagDisplayOracle extends GhidraScript{
 static final long F=0x5a1d5eL,Q=0x62dc50L,S=0x7fff0000L,RET=0x12345000L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 int hash(EmulatorHelper e,int n){int h=0x811c9dc5;if(n>0)for(byte b:e.readMemory(toAddr(Q),n))h=(h^(b&255))*16777619;return h;}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.setBreakpoint(toAddr(RET));put(e,0x622cb8,4,0);put(e,0x498c24,2,1024);put(e,0x498c20,2,1024);put(e,0x622d54,2,400);put(e,0x622d56,2,200);put(e,0x622d58,2,400);put(e,0x622d5a,2,700);
  for(int s=0;s<8192;s++){int transition=s&255,color=new int[]{0,1,3,4,7,127,128,255}[s/256%8],phase=s/2048%4,mode=new int[]{0,1,15,16,-1}[s*7%5],x=new int[]{-17,-16,500,1024,1025}[s*3%5],y=new int[]{-17,-16,500,1024,1025}[(s*2+1)%5],a=new int[]{0,2,255}[s/7%3],b=new int[]{0,2,255}[s/11%3];
   for(int i=0;i<64;i++)put(e,F+i*6,2,-1);e.writeMemory(toAddr(Q),new byte[48]);put(e,F,2,x);put(e,F+2,2,y);put(e,F+4,1,color);put(e,F+5,1,transition);put(e,0x622d40,2,phase);put(e,0x622cc0,4,mode);put(e,0x622e18,1,a);put(e,0x622e19,1,b);put(e,0x62b3a4,4,Q);put(e,S,4,RET);e.writeRegister("ESP",S);if(!e.run(toAddr(0x41d110),null,monitor))throw new Exception(e.getLastError());if(e.readRegister("EIP").longValue()!=RET)throw new Exception("unexpected caller");int size=(int)(read(e,0x62b3a4,4)-Q);out.println(s+","+transition+","+color+","+phase+","+mode+","+x+","+y+","+a+","+b+","+read(e,F+5,1)+","+size+","+(hash(e,size)&0xffffffffL));
  }
 }finally{e.dispose();}}}
}
