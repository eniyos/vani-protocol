/**
 * Vani SDK Test File
 * 
 * Run with: node test.js
 */

import VaniSDK from './index.js';

async function test() {
  console.log('🚀 Testing Vani SDK...\n');

  // Initialize SDK
  const vani = new VaniSDK({
    apiURL: 'http://localhost:8080'
  });

  try {
    // Test 1: Health check
    console.log('1️⃣ Testing health check...');
    const health = await vani.health();
    console.log(`   Health: ${health}\n`);

    // Test 2: Parse command
    console.log('2️⃣ Testing parse command...');
    const parseResult = await vani.parseCommand('1 SOL se USDC swap karo', 'hindi');
    console.log('   Parse result:', JSON.stringify(parseResult, null, 2));
    console.log(`   Action: ${parseResult.action}`);
    console.log(`   Confidence: ${parseResult.confidence}\n`);

    // Test 3: Execute transaction
    console.log('3️⃣ Testing execute transaction...');
    const executeResult = await vani.executeTransaction(parseResult);
    console.log('   Execute result:', JSON.stringify(executeResult, null, 2));
    console.log(`   Success: ${executeResult.success}`);
    console.log(`   Transaction ID: ${executeResult.transaction_id}\n`);

    // Test 4: Combined parse and execute
    console.log('4️⃣ Testing combined parse and execute...');
    const combinedResult = await vani.parseAndExecute('2 USDC se SOL swap karo', 'hindi');
    console.log('   Combined result:', JSON.stringify(combinedResult, null, 2));
    console.log(`   Success: ${combinedResult.success}\n`);

    console.log('✅ All tests passed!');
  } catch (error) {
    console.error('❌ Test failed:', error.message);
    process.exit(1);
  }
}

test();