//Real20/73 ->44ad40 ->56840 ->448c20, including native RNG and59af0.
//Only libm sqrt and the downstream464a20 projectile allocator are isolated.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
import java.math.BigInteger;
import java.nio.file.*;
public class ZNativePointFlameOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,T=0x1300000L;
 long[]shot;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void call(EmulatorHelper e,long f)throws Exception{put(e,S,4,RET);put(e,S+4,4,U);e.writeRegister("ESP",S);e.writeRegister("EIP",f);e.getEmulator().setHalt(false);for(int n=0;n<100000;n++){long pc=e.readRegister("EIP").longValue();if(pc==RET)return;if(pc==0x473264||pc==0x464a20){long sp=e.readRegister("ESP").longValue();if(pc==0x473264){double x=Double.longBitsToDouble(e.readRegister("XMM0").longValue());e.writeRegister("XMM0",BigInteger.valueOf(Double.doubleToRawLongBits(Math.sqrt(x))));}else{shot=new long[5];for(int i=0;i<5;i++)shot[i]=read(e,sp+4+i*4,4);}e.writeRegister("EIP",read(e,sp,4));e.writeRegister("ESP",sp+4);continue;}if(!e.step(monitor))throw new Exception(e.getLastError());}throw new Exception("CPU budget at "+e.readRegister("EIP").toString(16));}
 public void run()throws Exception{String base=System.getenv("Z_GAME_DIR");if(base==null)base="C:/GOG Games/Z";byte[]trig=Files.readAllBytes(Paths.get(base,"sincos.dat"));try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{for(int i=0;i<640;i++){short v=(short)(((trig[i*2]&255)<<8)|(trig[i*2+1]&255));put(e,T+i*2,2,v>>5);}put(e,0x586a80,4,T);
 for(int phase:new int[]{20,73})for(int raw:new int[]{-32768,-123,-1,0,1,0x5500,0x5501,0x5502,239,0x55ef})for(int d=0;d<8;d++)for(int aim:new int[]{0,60,120,255})for(int counter=0;counter<2;counter++)for(int mp=0;mp<2;mp++){
  e.writeMemory(toAddr(U),new byte[240*192]);put(e,U+192+44,2,500+(d%2==0?40:400));put(e,U+192+46,2,570+d*7);put(e,U+192+124,1,0);put(e,U+384+44,2,680);put(e,U+384+46,2,510);put(e,U+384+124,1,2);put(e,U+239*192+44,2,1300);put(e,U+239*192+46,2,500);put(e,U+239*192+124,1,1);
  put(e,U+44,2,500);put(e,U+46,2,600);put(e,U+2,1,aim);put(e,U+4,1,30);put(e,U+0x35,1,phase);put(e,U+0x32,2,phase==20?3:0);put(e,U+0x3c,1,phase==20?1:0);put(e,U+0x39,1,d);put(e,U+0x42,2,11);put(e,U+0x44,2,22);put(e,U+0x46,2,raw);put(e,U+0x50,4,300+d*10);put(e,U+0x54,2,400+counter*3);put(e,U+0x6a,2,300+d*10);put(e,U+0x6c,2,400+counter*3);put(e,U+0x69,1,d%2==0?0:128);put(e,U+0x78,1,98);put(e,U+0x79,1,13);put(e,U+0x7c,1,15);put(e,U+0x97,1,counter);put(e,0x622dc4,1,mp);put(e,0x5860c6,1,1);for(int j=0;j<5;j++)put(e,0x622cdc+j*4,4,j+1);shot=null;call(e,phase==20?0x44af30:0x44b080);if(shot==null)throw new Exception("No shot");StringBuilder row=new StringBuilder(phase+","+raw+","+d+","+aim+","+counter+","+mp+","+read(e,U+0x32,2)+","+(byte)read(e,U+0x3c,1)+","+read(e,U+0x7b,1)+","+(short)read(e,U+0x46,2)+","+read(e,U+0x39,1)+","+(short)read(e,U+0x42,2)+","+(short)read(e,U+0x44,2));for(long n:shot)row.append(',').append(n);for(int j=0;j<5;j++)row.append(',').append(read(e,0x622cdc+j*4,4));out.println(row);
 }out.flush();}finally{e.dispose();}}}
}
