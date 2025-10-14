-- =============================================================================
-- 果蔬智造测试数据
-- 创建时间: 2025-10-14
-- 说明: 插入测试数据用于开发和测试
-- =============================================================================

-- =============================================================================
-- 1. 插入测试用户
-- =============================================================================
DO $$
DECLARE
    v_admin_id UUID;
    v_user1_id UUID;
    v_user2_id UUID;
BEGIN
    -- 插入管理员用户
    INSERT INTO users (id, username, email, password_hash, full_name, role, status)
    VALUES (
        gen_random_uuid(),
        'admin',
        'admin@guoyan.com',
        '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYuPHZ1234',  -- password: admin123
        '系统管理员',
        'admin',
        'active'
    ) RETURNING id INTO v_admin_id;

    -- 插入测试用户1
    INSERT INTO users (id, username, email, password_hash, full_name, role, status)
    VALUES (
        gen_random_uuid(),
        'zhangsan',
        'zhangsan@example.com',
        '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYuPHZ1234',  -- password: user123
        '张三',
        'user',
        'active'
    ) RETURNING id INTO v_user1_id;

    -- 插入测试用户2
    INSERT INTO users (id, username, email, password_hash, full_name, role, status)
    VALUES (
        gen_random_uuid(),
        'lisi',
        'lisi@example.com',
        '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYuPHZ1234',  -- password: user123
        '李四',
        'operator',
        'active'
    ) RETURNING id INTO v_user2_id;

    RAISE NOTICE '已创建测试用户: admin_id=%, user1_id=%, user2_id=%', v_admin_id, v_user1_id, v_user2_id;
END $$;

-- =============================================================================
-- 2. 插入上传记录和检测结果
-- =============================================================================
DO $$
DECLARE
    v_upload_id UUID;
    v_user_id UUID;
    v_filename TEXT;
    v_status TEXT;
    v_total_fruits INTEGER;
    v_grade_a INTEGER;
    v_grade_b INTEGER;
    v_grade_c INTEGER;
    v_avg_confidence REAL;
    v_detections JSONB;
    v_created_at TIMESTAMPTZ;
    i INTEGER;
    j INTEGER;
    det_count INTEGER;
    fruit_types TEXT[] := ARRAY['苹果', '香蕉', '橙子', '葡萄', '西瓜', '草莓', '樱桃', '梨', '桃子', '芒果'];
    grades TEXT[] := ARRAY['A', 'B', 'C'];
    confidence REAL;
    grade TEXT;
    class_name TEXT;
    user_ids UUID[];
BEGIN
    -- 获取所有用户ID
    SELECT array_agg(id) INTO user_ids FROM users WHERE role != 'admin';

    -- 生成50条上传记录
    FOR i IN 1..50 LOOP
        v_upload_id := gen_random_uuid();
        v_user_id := user_ids[1 + floor(random() * array_length(user_ids, 1))::INTEGER];
        v_filename := 'test_image_' || i || '.jpg';
        v_status := CASE
            WHEN i % 10 = 0 THEN 'failed'
            WHEN i % 15 = 0 THEN 'processing'
            ELSE 'completed'
        END;

        -- 随机生成检测到的水果数量（5-20个）
        det_count := 5 + floor(random() * 15)::INTEGER;

        v_grade_a := 0;
        v_grade_b := 0;
        v_grade_c := 0;
        v_total_fruits := det_count;
        v_avg_confidence := 0;

        -- 生成检测结果JSON
        v_detections := '[]'::JSONB;

        FOR j IN 1..det_count LOOP
            -- 随机选择水果类型
            class_name := fruit_types[1 + floor(random() * array_length(fruit_types, 1))::INTEGER];

            -- 随机生成置信度 (0.70 - 0.99)
            confidence := 0.70 + (random() * 0.29);
            v_avg_confidence := v_avg_confidence + confidence;

            -- 随机选择等级
            grade := grades[1 + floor(random() * array_length(grades, 1))::INTEGER];

            -- 统计各等级数量
            IF grade = 'A' THEN
                v_grade_a := v_grade_a + 1;
            ELSIF grade = 'B' THEN
                v_grade_b := v_grade_b + 1;
            ELSE
                v_grade_c := v_grade_c + 1;
            END IF;

            -- 添加检测记录到JSON数组
            v_detections := v_detections || jsonb_build_object(
                'class_name', class_name,
                'confidence', confidence,
                'quality_grade', grade,
                'bbox', jsonb_build_array(
                    floor(random() * 500)::INTEGER,  -- x
                    floor(random() * 500)::INTEGER,  -- y
                    50 + floor(random() * 100)::INTEGER,  -- width
                    50 + floor(random() * 100)::INTEGER   -- height
                )
            );
        END LOOP;

        -- 计算平均置信度
        v_avg_confidence := v_avg_confidence / det_count;

        -- 设置创建时间（过去30天内随机）
        v_created_at := NOW() - (random() * INTERVAL '30 days');

        -- 插入上传记录（使用小的占位符数据）
        INSERT INTO uploads (id, user_id, filename, content_type, data, status, created_at, updated_at)
        VALUES (
            v_upload_id,
            v_user_id,
            v_filename,
            'image/jpeg',
            decode('FFD8FFE000104A46494600', 'hex'), -- JPEG 文件头占位符
            v_status,
            v_created_at,
            v_created_at
        );

        -- 如果状态是completed，插入结果记录
        IF v_status = 'completed' THEN
            INSERT INTO results (upload_id, status, detections, total_fruits, grade_a, grade_b, grade_c, avg_confidence, created_at)
            VALUES (
                v_upload_id,
                'completed',
                v_detections,
                v_total_fruits,
                v_grade_a,
                v_grade_b,
                v_grade_c,
                v_avg_confidence,
                v_created_at
            );
        END IF;
    END LOOP;
END $$;

-- =============================================================================
-- 3. 插入实时检测记录
-- =============================================================================
DO $$
DECLARE
    v_user_id UUID;
    v_labels TEXT[];
    v_confidence_scores REAL[];
    v_detected_classes TEXT[];
    v_created_at TIMESTAMPTZ;
    i INTEGER;
    j INTEGER;
    det_count INTEGER;
    fruit_types TEXT[] := ARRAY['苹果', '香蕉', '橙子', '葡萄', '西瓜', '草莓', '樱桃', '梨', '桃子', '芒果'];
    sources TEXT[] := ARRAY['live_camera', 'camera_1', 'camera_2', 'mobile_app'];
    user_ids UUID[];
BEGIN
    -- 获取所有用户ID
    SELECT array_agg(id) INTO user_ids FROM users;

    FOR i IN 1..30 LOOP
        -- 随机选择用户
        v_user_id := user_ids[1 + floor(random() * array_length(user_ids, 1))::INTEGER];

        -- 随机生成检测到的物体数量（3-10个）
        det_count := 3 + floor(random() * 7)::INTEGER;

        v_labels := ARRAY[]::TEXT[];
        v_confidence_scores := ARRAY[]::REAL[];
        v_detected_classes := ARRAY[]::TEXT[];

        FOR j IN 1..det_count LOOP
            v_labels := array_append(v_labels, fruit_types[1 + floor(random() * array_length(fruit_types, 1))::INTEGER]);
            v_confidence_scores := array_append(v_confidence_scores, (0.70 + random() * 0.29)::REAL);
            v_detected_classes := array_append(v_detected_classes, fruit_types[1 + floor(random() * array_length(fruit_types, 1))::INTEGER]);
        END LOOP;

        -- 设置创建时间（过去7天内随机）
        v_created_at := NOW() - (random() * INTERVAL '7 days');

        -- 插入实时检测记录
        INSERT INTO detections (user_id, snapshot, labels, confidence_scores, detected_classes, created_at, source)
        VALUES (
            v_user_id,
            decode('FFD8FFE000104A46494600', 'hex'), -- JPEG 文件头占位符
            v_labels,
            v_confidence_scores,
            v_detected_classes,
            v_created_at,
            sources[1 + floor(random() * array_length(sources, 1))::INTEGER]
        );
    END LOOP;
END $$;

-- =============================================================================
-- 4. 插入聊天会话和消息
-- =============================================================================
DO $$
DECLARE
    v_session_id UUID;
    v_user_id UUID;
    user_ids UUID[];
    i INTEGER;
    j INTEGER;
    message_count INTEGER;
BEGIN
    -- 获取所有非管理员用户ID
    SELECT array_agg(id) INTO user_ids FROM users WHERE role != 'admin';

    -- 为每个用户创建2-3个会话
    FOREACH v_user_id IN ARRAY user_ids LOOP
        FOR i IN 1..2 LOOP
            v_session_id := gen_random_uuid();

            -- 插入会话
            INSERT INTO chat_sessions (id, user_id, title, status, created_at, updated_at)
            VALUES (
                v_session_id,
                v_user_id,
                CASE i
                    WHEN 1 THEN '如何提高识别准确率？'
                    WHEN 2 THEN '查询最近的检测记录'
                    ELSE '水果分级标准咨询'
                END,
                'active',
                NOW() - (random() * INTERVAL '7 days'),
                NOW() - (random() * INTERVAL '1 day')
            );

            -- 为每个会话插入3-8条消息
            message_count := 3 + floor(random() * 5)::INTEGER;
            FOR j IN 1..message_count LOOP
                INSERT INTO chat_messages (session_id, role, content, created_at)
                VALUES (
                    v_session_id,
                    CASE j % 2 WHEN 1 THEN 'user' ELSE 'assistant' END,
                    CASE j % 2
                        WHEN 1 THEN '用户问题 ' || j || ': 请帮我查询...'
                        ELSE 'AI回复 ' || j || ': 根据查询结果...'
                    END,
                    NOW() - (random() * INTERVAL '7 days') + (j * INTERVAL '5 minutes')
                );
            END LOOP;
        END LOOP;
    END LOOP;
END $$;

-- =============================================================================
-- 5. 统计信息
-- =============================================================================
DO $$
DECLARE
    user_count INTEGER;
    upload_count INTEGER;
    result_count INTEGER;
    detection_count INTEGER;
    session_count INTEGER;
    message_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO user_count FROM users;
    SELECT COUNT(*) INTO upload_count FROM uploads;
    SELECT COUNT(*) INTO result_count FROM results;
    SELECT COUNT(*) INTO detection_count FROM detections;
    SELECT COUNT(*) INTO session_count FROM chat_sessions;
    SELECT COUNT(*) INTO message_count FROM chat_messages;

    RAISE NOTICE '=== 数据库初始化完成 ===';
    RAISE NOTICE '用户数: %', user_count;
    RAISE NOTICE '上传记录数: %', upload_count;
    RAISE NOTICE '检测结果数: %', result_count;
    RAISE NOTICE '实时检测数: %', detection_count;
    RAISE NOTICE '聊天会话数: %', session_count;
    RAISE NOTICE '聊天消息数: %', message_count;
    RAISE NOTICE '========================';
END $$;
