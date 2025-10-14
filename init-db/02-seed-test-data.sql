-- 插入测试数据
-- 注意：这里使用小的占位符字节数据，实际环境中应该是真实的图片数据

-- 插入50条上传记录和对应的检测结果
DO $$
DECLARE
    v_upload_id UUID;
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
BEGIN
    -- 生成50条上传记录
    FOR i IN 1..50 LOOP
        v_upload_id := gen_random_uuid();
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
        INSERT INTO uploads (id, filename, content_type, data, status, created_at, updated_at)
        VALUES (
            v_upload_id,
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

-- 插入30条实时检测记录
DO $$
DECLARE
    v_labels TEXT[];
    v_confidence_scores REAL[];
    v_detected_classes TEXT[];
    v_created_at TIMESTAMPTZ;
    i INTEGER;
    j INTEGER;
    det_count INTEGER;
    fruit_types TEXT[] := ARRAY['苹果', '香蕉', '橙子', '葡萄', '西瓜', '草莓', '樱桃', '梨', '桃子', '芒果'];
    sources TEXT[] := ARRAY['live_camera', 'camera_1', 'camera_2', 'mobile_app'];
BEGIN
    FOR i IN 1..30 LOOP
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
        INSERT INTO detections (snapshot, labels, confidence_scores, detected_classes, created_at, source)
        VALUES (
            decode('FFD8FFE000104A46494600', 'hex'), -- JPEG 文件头占位符
            v_labels,
            v_confidence_scores,
            v_detected_classes,
            v_created_at,
            sources[1 + floor(random() * array_length(sources, 1))::INTEGER]
        );
    END LOOP;
END $$;

-- 输出统计信息
DO $$
DECLARE
    upload_count INTEGER;
    result_count INTEGER;
    detection_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO upload_count FROM uploads;
    SELECT COUNT(*) INTO result_count FROM results;
    SELECT COUNT(*) INTO detection_count FROM detections;

    RAISE NOTICE '=== 数据库初始化完成 ===';
    RAISE NOTICE '上传记录数: %', upload_count;
    RAISE NOTICE '检测结果数: %', result_count;
    RAISE NOTICE '实时检测数: %', detection_count;
    RAISE NOTICE '========================';
END $$;
